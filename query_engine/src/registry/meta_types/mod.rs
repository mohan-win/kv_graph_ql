//! Impements necessary meta-data types for introspection.
use crate::{introspection::types::__DirectiveLocation, Value};
use indexmap::{map::IndexMap, set::IndexSet};
use std::{fmt, sync::Arc};

pub mod from_service_doc;

fn strip_brackets(type_name: &str) -> Option<&str> {
  type_name
    .strip_prefix('[')
    .map(|rest| &rest[..rest.len() - 1])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaTypeName<'a> {
  List(&'a str),
  NonNull(&'a str),
  Named(&'a str),
}

impl<'a> fmt::Display for MetaTypeName<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      MetaTypeName::Named(name) => write!(f, "{}", name),
      MetaTypeName::NonNull(name) => write!(f, "{}!", name),
      MetaTypeName::List(name) => write!(f, "[{}]", name),
    }
  }
}

impl<'a> MetaTypeName<'a> {
  #[inline]
  pub fn create(type_name: &str) -> MetaTypeName {
    if let Some(type_name) = type_name.strip_suffix('!') {
      MetaTypeName::NonNull(type_name)
    } else if let Some(type_name) = strip_brackets(type_name) {
      MetaTypeName::List(type_name)
    } else {
      MetaTypeName::Named(type_name)
    }
  }

  #[inline]
  pub fn concrete_typename(type_name: &str) -> &str {
    match MetaTypeName::create(type_name) {
      MetaTypeName::List(type_name) => Self::concrete_typename(type_name),
      MetaTypeName::NonNull(type_name) => Self::concrete_typename(type_name),
      MetaTypeName::Named(type_name) => type_name,
    }
  }

  #[inline]
  pub fn is_non_null(&self) -> bool {
    matches!(self, MetaTypeName::NonNull(_))
  }

  #[inline]
  pub fn is_list(&self) -> bool {
    match self {
      Self::List(_) => true,
      Self::NonNull(ty) => MetaTypeName::create(ty).is_list(),
      Self::Named(ty) => ty.ends_with(']'),
    }
  }

  #[inline]
  #[must_use]
  pub fn unwrap_non_null(&self) -> Self {
    match self {
      MetaTypeName::NonNull(ty) => MetaTypeName::create(ty),
      _ => *self,
    }
  }

  pub fn is_subtype(&self, sub: &MetaTypeName<'_>) -> bool {
    match (self, sub) {
      (MetaTypeName::NonNull(super_type), MetaTypeName::NonNull(sub_type))
      | (MetaTypeName::Named(super_type), MetaTypeName::NonNull(sub_type)) => {
        MetaTypeName::create(super_type).is_subtype(&MetaTypeName::create(sub_type))
      }
      (MetaTypeName::Named(super_type), MetaTypeName::Named(sub_type)) => {
        super_type == sub_type
      }
      (MetaTypeName::List(super_type), MetaTypeName::List(sub_type)) => {
        MetaTypeName::create(super_type).is_subtype(&MetaTypeName::create(sub_type))
      }
      _ => false,
    }
  }
}

/// actual directive invocation on SDL definitions
#[derive(Debug, Clone)]
pub struct MetaDirectiveInvocation {
  /// name of the directive to invoke.
  pub name: String,
  /// actual arguments passed to directive.
  pub args: IndexMap<String, Value>,
}

/// Input value metadata.
#[derive(Clone)]
pub struct MetaInputValue {
  /// The name of the input value
  pub name: String,
  /// The description of the input value
  pub description: Option<String>,
  /// The type of the input value
  pub ty: String,
  /// The default value of the input value
  pub default_value: Option<String>,
  /// Custom directive invocations
  pub directive_invocations: Vec<MetaDirectiveInvocation>,
}

#[derive(Debug, Clone, Default)]
pub enum Deprecation {
  #[default]
  NoDeprecated,
  Deprecated {
    reason: Option<String>,
  },
}

impl Deprecation {
  pub fn is_deprecated(&self) -> bool {
    matches!(self, Deprecation::Deprecated { .. })
  }

  pub fn reason(&self) -> Option<&str> {
    match self {
      Self::NoDeprecated => None,
      Self::Deprecated { reason } => reason.as_deref(),
    }
  }
}

/// Field metadata.
#[derive(Clone)]
pub struct MetaField {
  /// The name of the field
  pub name: String,
  /// The description of the field
  pub description: Option<String>,
  /// The arguments of the field
  pub args: IndexMap<String, MetaInputValue>,
  /// The type of the field
  pub ty: String,
  /// Field deprecation
  pub deprecation: Deprecation,
  /// Custom directive invocations
  pub directive_invocations: Vec<MetaDirectiveInvocation>,
}

#[derive(Debug, Clone)]
pub struct MetaEnumValue {
  pub name: String,
  pub description: Option<String>,
  pub deprecation: Deprecation,
  pub directive_invocations: Vec<MetaDirectiveInvocation>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum MetaTypeId {
  Scalar,
  Object,
  Interface,
  Union,
  Enum,
  InputObject,
}

impl fmt::Display for MetaTypeId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(match self {
      MetaTypeId::Scalar => "Scalar",
      MetaTypeId::Object => "Object",
      MetaTypeId::Interface => "Interface",
      MetaTypeId::Union => "Union",
      MetaTypeId::Enum => "Enum",
      MetaTypeId::InputObject => "InputObject",
    })
  }
}

/// A validator for scalar
pub type ScalarValidatorFn = Arc<dyn Fn(&Value) -> bool + Send + Sync>;

/// Type metadata.
#[derive(Clone)]
pub enum MetaType {
  /// Scalar
  ///
  /// Reference: <https://spec.graphql.org/October2021/#sec-Scalars>
  Scalar {
    /// The name of the scalar
    name: String,
    /// the description of the scalar
    description: Option<String>,
    /// A function that uses to check if the scalar is valid
    is_valid: Option<ScalarValidatorFn>,
    /// Provide a specification URL for this scalar type, it must link to a
    /// human-readable specification of the data format, serialization and
    /// coercion rules for this scalar.
    specified_by_url: Option<String>,
  },
  /// Object
  ///
  /// Reference: <https://spec.graphql.org/October2021/#sec-Objects>
  Object {
    /// The name of the object.
    name: String,
    /// The description of the object.
    description: Option<String>,
    /// Interfaces implemented by this object.
    implements: IndexSet<String>,
    /// The fields of the object type.
    fields: IndexMap<String, MetaField>,
    /// Indicates whether it is a subscription object.
    is_subscription: bool,
    /// Custom directive invocations.
    directive_invocations: Vec<MetaDirectiveInvocation>,
  },
  /// Interface
  ///
  /// Reference: <https://spec.graphql.org/October2021/#sec-Interfaces>
  Interface {
    /// The name of the interface
    name: String,
    /// The description of the interface
    description: Option<String>,
    /// Interfaces implemented by this interface.
    implements: IndexSet<String>,
    /// The fields of the interface
    fields: IndexMap<String, MetaField>,
    /// custom directive invocations
    directive_invocations: Vec<MetaDirectiveInvocation>,
  },
  /// Union
  ///
  /// Reference: <https://spec.graphql.org/October2021/#sec-Unions>
  Union {
    /// The name of the interface
    name: String,
    /// The description of the union
    description: Option<String>,
    /// The object types that could be the union
    possible_types: IndexSet<String>,
  },
  Enum {
    /// The name of the enum
    name: String,
    /// The description of the enum
    description: Option<String>,
    /// The values of the enum
    enum_values: IndexMap<String, MetaEnumValue>,
    /// custom directive invocations
    directive_invocations: Vec<MetaDirectiveInvocation>,
  },
  /// Input object
  ///
  /// Reference: <https://spec.graphql.org/October2021/#sec-Input-Objects>
  InputObject {
    /// The name of the input object
    name: String,
    /// The description of the input object
    description: Option<String>,
    /// The fields of the input object
    input_fields: IndexMap<String, MetaInputValue>,
    /// Is the oneof input objects
    ///
    /// Reference: <https://github.com/graphql/graphql-spec/pull/825>
    oneof: bool,
    /// custom directive invocations
    directive_invocations: Vec<MetaDirectiveInvocation>,
  },
}

impl MetaType {
  #[inline]
  pub fn type_id(&self) -> MetaTypeId {
    match self {
      Self::Scalar { .. } => MetaTypeId::Scalar,
      Self::Object { .. } => MetaTypeId::Object,
      Self::Interface { .. } => MetaTypeId::Interface,
      Self::Union { .. } => MetaTypeId::Union,
      Self::Enum { .. } => MetaTypeId::Enum,
      Self::InputObject { .. } => MetaTypeId::InputObject,
    }
  }

  #[inline]
  pub fn fields(&self) -> Option<&IndexMap<String, MetaField>> {
    match self {
      MetaType::Object { fields, .. } => Some(&fields),
      MetaType::Interface { fields, .. } => Some(&fields),
      _ => None,
    }
  }

  #[inline]
  pub fn field_by_name(&self, name: &str) -> Option<&MetaField> {
    self.fields().and_then(|fields| fields.get(name))
  }

  #[inline]
  pub fn name(&self) -> &str {
    match self {
      Self::Scalar { name, .. } => name,
      Self::Object { name, .. } => name,
      Self::Interface { name, .. } => name,
      Self::Union { name, .. } => name,
      Self::Enum { name, .. } => name,
      Self::InputObject { name, .. } => name,
    }
  }

  #[inline]
  pub fn is_abstract(&self) -> bool {
    matches!(self, Self::Interface { .. } | Self::Union { .. })
  }

  #[inline]
  pub fn is_leaf(&self) -> bool {
    matches!(self, Self::Enum { .. } | Self::Scalar { .. })
  }

  #[inline]
  pub fn is_input(&self) -> bool {
    matches!(
      self,
      Self::Enum { .. } | Self::Scalar { .. } | Self::InputObject { .. }
    )
  }
}

pub struct MetaDirective {
  pub name: String,
  pub description: Option<String>,
  pub locations: Vec<__DirectiveLocation>,
  pub args: IndexMap<String, MetaInputValue>,
  pub is_repeatable: bool,
}
