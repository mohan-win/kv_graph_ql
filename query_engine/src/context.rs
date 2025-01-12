use crate::{
  graphql_parser::{
    types::{Directive, Field, FragmentDefinition, OperationDefinition, SelectionSet},
    Pos, Positioned,
  },
  graphql_value::{ConstValue, Name, Variables},
  schema::{IntrospectionMode, SchemaEnv},
  Error, InputType, PathSegment, Result, ServerError, ServerResult,
};
use fnv::FnvHashMap;
use graphql_value::Value;
use http;
use serde::{ser::SerializeSeq, Serialize};
use std::{
  any::{Any, TypeId},
  collections::HashMap,
  fmt::{Debug, Display},
  ops::Deref,
  sync::{Arc, Mutex},
};

/// Data related functions of the context.
pub trait DataContext<'a> {
  /// Gets the global data defined in the `Context` or `Schema`.
  ///
  /// If both `Schema` and `Query` have the same data type, the data in the
  /// `Query` is obtained.
  ///
  /// # Errors
  /// Returns a `Error` if the specified type data doesn't exists.
  fn data<D: Any + Send + Sync>(&self) -> Result<&'a D>;

  /// Gets the global data defined in the `Context` or `Schema`.
  ///
  /// # Panics
  /// It will panic if the specified data type doesn't exists.
  fn data_unchecked<D: Any + Send + Sync>(&self) -> &'a D;

  /// Gets the global data defined in the `Context` or `Schema` or `None` if
  /// the specified data type is not present.
  fn data_opt<D: Any + Send + Sync>(&self) -> Option<&'a D>;
}

/// Context data.
///
/// This is a type map, allowing you to store anything inside it.
#[derive(Default)]
pub struct Data(FnvHashMap<TypeId, Box<dyn Any + Send + Sync>>);

impl Deref for Data {
  type Target = FnvHashMap<TypeId, Box<dyn Any + Send + Sync>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Data {
  pub fn insert<D: Any + Send + Sync>(&mut self, data: D) {
    self.0.insert(TypeId::of::<D>(), Box::new(data));
  }

  pub(crate) fn merge(&mut self, other: Data) {
    self.0.extend(other.0);
  }
}

impl Debug for Data {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("Data").finish()
  }
}

/// A segment in the path to the current query.
///
/// This is borrowed form of [`PathSegment`](enum.PathSegment.html) used during
/// execution instead of passed back when error occurs.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(untagged)]
pub enum QueryPathSegment<'a> {
  /// We are currently resolving an element in a list.
  Index(usize),
  /// We are currently resolving a field in an object.
  Name(&'a str),
}

/// A path to the current query.
///
/// The path is stored as a kind of reverse linked list
#[derive(Debug, Clone, Copy)]
pub struct QueryPathNode<'a> {
  /// The parent node to this, if there is one.
  pub parent: Option<&'a QueryPathNode<'a>>,

  /// The current path segment being resolved.
  pub segment: QueryPathSegment<'a>,
}

impl<'a> serde::Serialize for QueryPathNode<'a> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut seq = serializer.serialize_seq(None)?;
    self.try_for_each(|segment| seq.serialize_element(segment))?;
    seq.end()
  }
}

impl<'a> Display for QueryPathNode<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut first = true;
    self.try_for_each(|segment| {
      if !first {
        write!(f, ".")?;
      }
      first = false;

      match segment {
        QueryPathSegment::Index(idx) => write!(f, "{}", *idx),
        QueryPathSegment::Name(name) => write!(f, "{}", name),
      }
    })
  }
}

impl<'a> QueryPathNode<'a> {
  /// Get the current field name.
  ///
  /// This traverses all the parents of the node until it finds one that is a
  /// field name.
  pub fn field_name(&self) -> &str {
    std::iter::once(self)
      .chain(self.parents())
      .find_map(|node| match node.segment {
        QueryPathSegment::Name(name) => Some(name),
        QueryPathSegment::Index(_) => None,
      })
      .unwrap()
  }

  /// Get the path represented by `Vec<String>`; numbers will be stringified.
  #[must_use]
  pub fn to_string_vec(self) -> Vec<String> {
    let mut res = Vec::new();
    self.for_each(|s| {
      res.push(match s {
        QueryPathSegment::Name(name) => (*name).to_string(),
        QueryPathSegment::Index(idx) => idx.to_string(),
      });
    });
    res
  }

  pub fn parents(&self) -> Parents<'_> {
    Parents(self)
  }

  pub(crate) fn for_each<F: FnMut(&QueryPathSegment<'a>)>(&self, mut f: F) {
    let _ = self.try_for_each::<std::convert::Infallible, _>(|segment| {
      f(segment);
      Ok(())
    });
  }

  pub(crate) fn try_for_each<E, F: FnMut(&QueryPathSegment<'a>) -> Result<(), E>>(
    &self,
    mut f: F,
  ) -> Result<(), E> {
    self.try_for_each_ref(&mut f)
  }

  fn try_for_each_ref<E, F: FnMut(&QueryPathSegment<'a>) -> Result<(), E>>(
    &self,
    f: &mut F,
  ) -> Result<(), E> {
    if let Some(parent) = &self.parent {
      parent.try_for_each_ref(f)?;
    }
    f(&self.segment)
  }
}

/// An iterator over the parents of a
/// QueryPathNode
pub struct Parents<'a>(&'a QueryPathNode<'a>);

impl<'a> Parents<'a> {
  /// Get the current query path node, which the call to `next` will
  /// get the parents of.
  pub fn current(&self) -> &'a QueryPathNode<'a> {
    self.0
  }
}

impl<'a> Iterator for Parents<'a> {
  type Item = &'a QueryPathNode<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    let parent = self.0.parent;
    if let Some(parent) = parent {
      self.0 = parent;
    }
    parent
  }
}

impl<'a> std::iter::FusedIterator for Parents<'a> {}

/// Context for `SelectionSet`
pub type ContextSelectionSet<'a> = ContextBase<'a, &'a Positioned<SelectionSet>>;

/// Context for resolve field.
pub type Context<'a> = ContextBase<'a, &'a Positioned<Field>>;

/// Context for execute directive.
pub type ContextDirective<'a> = ContextBase<'a, &'a Positioned<Directive>>;

/// Query context
pub struct ContextBase<'a, T> {
  /// The current path node being resolved.
  pub path_node: Option<QueryPathNode<'a>>,
  /// If `true` means the current field is for introspection.
  pub(crate) is_for_introspection: bool,
  #[doc(hidden)]
  pub item: T,
  #[doc(hidden)]
  pub schema_env: &'a SchemaEnv,
  #[doc(hidden)]
  pub query_env: &'a QueryEnv,
  #[doc(hidden)]
  pub execute_data: Option<&'a Data>,
}

#[doc(hidden)]
pub struct QueryEnvInner {
  pub variables: Variables,
  pub operation_name: Option<String>,
  pub operation: Positioned<OperationDefinition>,
  pub fragments: HashMap<Name, Positioned<FragmentDefinition>>,
  pub session_data: Arc<Data>,
  pub query_data: Arc<Data>,
  pub http_headers: Mutex<http::HeaderMap>,
  pub introspection_mode: IntrospectionMode,
  pub errors: Mutex<Vec<ServerError>>,
}

#[doc(hidden)]
#[derive(Clone)]
pub struct QueryEnv(pub(crate) Arc<QueryEnvInner>);

impl Deref for QueryEnv {
  type Target = QueryEnvInner;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl QueryEnv {
  #[doc(hidden)]
  pub fn new(inner: QueryEnvInner) -> QueryEnv {
    QueryEnv(Arc::new(inner))
  }

  #[doc(hidden)]
  pub fn create_context<'a, T>(
    &'a self,
    schema_env: &'a SchemaEnv,
    path_node: Option<QueryPathNode<'a>>,
    item: T,
    execute_data: Option<&'a Data>,
  ) -> ContextBase<'a, T> {
    ContextBase {
      path_node,
      is_for_introspection: false,
      item,
      schema_env,
      query_env: self,
      execute_data,
    }
  }
}

impl<'a, T> DataContext<'a> for ContextBase<'a, T> {
  fn data<D: Any + Send + Sync>(&self) -> Result<&'a D> {
    ContextBase::data::<D>(self)
  }

  fn data_unchecked<D: Any + Send + Sync>(&self) -> &'a D {
    ContextBase::data_unchecked::<D>(self)
  }

  fn data_opt<D: Any + Send + Sync>(&self) -> Option<&'a D> {
    ContextBase::data_opt::<D>(self)
  }
}

impl<'a, T> ContextBase<'a, T> {
  #[doc(hidden)]
  pub fn with_field(
    &'a self,
    field: &'a Positioned<Field>,
  ) -> ContextBase<'a, &'a Positioned<Field>> {
    ContextBase {
      path_node: Some(QueryPathNode {
        parent: self.path_node.as_ref(),
        segment: QueryPathSegment::Name(&field.node.response_key().node),
      }),
      is_for_introspection: self.is_for_introspection,
      item: field,
      schema_env: &self.schema_env,
      query_env: &self.query_env,
      execute_data: self.execute_data,
    }
  }

  #[doc(hidden)]
  pub fn with_selection_set(
    &'a self,
    selection_set: &'a Positioned<SelectionSet>,
  ) -> ContextBase<'a, &'a Positioned<SelectionSet>> {
    ContextBase {
      path_node: self.path_node,
      is_for_introspection: self.is_for_introspection,
      item: selection_set,
      schema_env: self.schema_env,
      query_env: self.query_env,
      execute_data: self.execute_data,
    }
  }

  #[doc(hidden)]
  #[must_use]
  pub fn with_index(&'a self, idx: usize) -> ContextBase<'a, T>
  where
    T: Copy,
  {
    ContextBase {
      path_node: Some(QueryPathNode {
        parent: self.path_node.as_ref(),
        segment: QueryPathSegment::Index(idx),
      }),
      is_for_introspection: self.is_for_introspection,
      item: self.item,
      schema_env: self.schema_env,
      query_env: self.query_env,
      execute_data: self.execute_data,
    }
  }

  #[doc(hidden)]
  pub fn set_error_path(&self, error: ServerError) -> ServerError {
    if let Some(node) = self.path_node {
      let mut path = Vec::new();
      node.for_each(|current_node| {
        path.push(match current_node {
          QueryPathSegment::Name(name) => PathSegment::Field((*name).to_string()),
          QueryPathSegment::Index(idx) => PathSegment::Index(*idx),
        });
      });
      ServerError { path, ..error }
    } else {
      error
    }
  }

  /// Report a resolver error.
  ///
  /// When implementing `OutputType`, if an error occurs, call this function
  /// to report this error and return `Value::Null`.
  pub fn add_error(&self, error: ServerError) {
    self.query_env.errors.lock().unwrap().push(error);
  }

  /// Gets the global data defined in the `Context` or `Schema`.
  ///
  /// If both `Schema` and `Query` have the same data type, the data in the
  /// `Query` is obtained.
  pub fn data<D: Any + Send + Sync>(&self) -> Result<&'a D> {
    self.data_opt::<D>().ok_or_else(|| {
      Error::new(format!(
        "Data `{}` does not exists",
        std::any::type_name::<D>()
      ))
    })
  }

  /// Gets the global data defined in the `Context` or `Schema`.
  ///
  /// # Panics
  ///
  /// It will panic if the specified data doesn't exists
  pub fn data_unchecked<D: Any + Send + Sync>(&self) -> &'a D {
    self
      .data_opt::<D>()
      .unwrap_or_else(|| panic!("Data `{}` does not exists", std::any::type_name::<D>()))
  }

  /// Gets the global data defined in the `Context` or `Schema` or `None` if
  /// the specified data does not exists.
  pub fn data_opt<D: Any + Send + Sync>(&self) -> Option<&'a D> {
    self
      .execute_data
      .as_ref()
      .and_then(|execute_data| execute_data.get(&TypeId::of::<D>()))
      .or_else(|| self.query_env.query_data.0.get(&TypeId::of::<D>()))
      .or_else(|| self.query_env.session_data.0.get(&TypeId::of::<D>()))
      .or_else(|| self.schema_env.data.0.get(&TypeId::of::<D>()))
      .and_then(|d| d.downcast_ref::<D>())
  }

  /// Returns whether the HTTP header `key` is  currently sent on the response.
  pub fn http_header_contains(&self, key: impl http::header::AsHeaderName) -> bool {
    self
      .query_env
      .http_headers
      .lock()
      .unwrap()
      .contains_key(key)
  }

  /// Sets http header to response.
  pub fn insert_http_header(
    &self,
    name: impl http::header::IntoHeaderName,
    value: impl TryInto<http::HeaderValue>,
  ) -> Option<http::HeaderValue> {
    if let Ok(value) = value.try_into() {
      self
        .query_env
        .http_headers
        .lock()
        .unwrap()
        .insert(name, value)
    } else {
      None
    }
  }

  /// Appends `value` to existing http header of given `name` in the response.
  /// Returns `false` if the header doesn't exists hence append fails.
  pub fn append_http_header(
    &self,
    name: impl http::header::IntoHeaderName,
    value: impl TryInto<http::HeaderValue>,
  ) -> bool {
    if let Ok(value) = value.try_into() {
      self
        .query_env
        .http_headers
        .lock()
        .unwrap()
        .append(name, value)
    } else {
      false
    }
  }

  fn var_value(&self, name: &str, pos: Pos) -> ServerResult<ConstValue> {
    self
      .query_env
      .operation
      .node
      .variable_definitions
      .iter()
      .find(|def| def.node.name.node == name)
      .and_then(|def| {
        self
          .query_env
          .variables
          .get(&def.node.name.node)
          .or_else(|| def.node.default_value())
      })
      .cloned()
      .ok_or_else(|| {
        ServerError::new(format!("Variable {} is not defined.", name), Some(pos))
      })
  }

  pub(crate) fn resolve_input_value(
    &self,
    value: Positioned<Value>,
  ) -> ServerResult<ConstValue> {
    let pos = value.pos;
    value
      .node
      .into_const_with(|name| self.var_value(&name, pos))
  }

  fn get_param_value<Q: InputType>(
    &self,
    arguments: &[(Positioned<Name>, Positioned<Value>)],
    name: &str,
    default: Option<fn() -> Q>,
  ) -> ServerResult<(Pos, Q)> {
    let value = arguments
      .iter()
      .find(|(n, _)| n.node.as_str() == name)
      .map(|(_, value)| value)
      .cloned();
    if value.is_none() {
      if let Some(default) = default {
        return Ok((Pos::default(), default()));
      }
    }

    let (pos, value) = match value {
      Some(value) => (value.pos, Some(self.resolve_input_value(value)?)),
      None => (Pos::default(), None),
    };

    InputType::parse(value)
      .map(|value| (pos, value))
      .map_err(|e| e.into_server_error(pos))
  }
}

impl<'a> ContextBase<'a, &'a Positioned<Field>> {
  #[doc(hidden)]
  pub fn param_value<T: InputType>(
    &self,
    name: &str,
    default: Option<fn() -> T>,
  ) -> ServerResult<(Pos, T)> {
    self.get_param_value(&self.item.node.arguments, name, default)
  }
}

impl<'a> ContextBase<'a, &'a Positioned<Directive>> {
  pub fn param_value<T: InputType>(
    &self,
    name: &str,
    default: Option<fn() -> T>,
  ) -> ServerResult<(Pos, T)> {
    self.get_param_value(&self.item.node.arguments, name, default)
  }
}
