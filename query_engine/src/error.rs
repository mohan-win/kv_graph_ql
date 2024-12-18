use std::{
  any::Any,
  collections::BTreeMap,
  fmt::{Debug, Display},
  marker::PhantomData,
  sync::Arc,
};

use crate::graphql_parser::{self as parser, Pos};
use crate::{graphql_value::ConstValue as Value, InputType};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Extensions to the error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct ErrorExtensionValues(BTreeMap<String, Value>);

impl ErrorExtensionValues {
  /// Set an extension value.
  pub fn set(&mut self, name: impl AsRef<str>, value: impl Into<Value>) {
    self.0.insert(name.as_ref().to_string(), value.into());
  }

  /// Unset an extension value.
  pub fn unset(&mut self, name: impl AsRef<str>) {
    self.0.remove(name.as_ref());
  }

  pub fn get(&self, name: impl AsRef<str>) -> Option<&Value> {
    self.0.get(name.as_ref())
  }
}

/// An error in GraphQL server.
#[derive(Clone, Serialize, Deserialize)]
pub struct ServerError {
  /// An explanatory message of the error.
  pub message: String,
  #[serde(skip)]
  pub source: Option<Arc<dyn Any + Send + Sync>>,
  /// Where the error occured.
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub locations: Vec<Pos>,
  /// If the error occurred in a resolver, the path to the error.
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub path: Vec<PathSegment>,
  /// Extensions to the error.
  #[serde(skip_serializing_if = "error_extensions_is_empty", default)]
  pub extensions: Option<ErrorExtensionValues>,
}

fn error_extensions_is_empty(values: &Option<ErrorExtensionValues>) -> bool {
  values.as_ref().map_or(true, |values| values.0.is_empty())
}

impl Debug for ServerError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ServerError")
      .field("message", &self.message)
      .field("locations", &self.locations)
      .field("path", &self.path)
      .field("extensions", &self.extensions)
      .finish()
  }
}

impl PartialEq for ServerError {
  fn eq(&self, other: &Self) -> bool {
    self.message.eq(&other.message)
      && self.locations.eq(&other.locations)
      && self.path.eq(&other.path)
      && self.extensions.eq(&other.extensions)
  }
}

impl ServerError {
  /// Create a new server error with the message.
  pub fn new(message: impl Into<String>, pos: Option<Pos>) -> Self {
    Self {
      message: message.into(),
      source: None,
      locations: pos.map(|pos| vec![pos]).unwrap_or_default(),
      path: Vec::new(),
      extensions: None,
    }
  }

  /// Get the source of the error.
  pub fn source<T: Any + Send + Sync>(&self) -> Option<&T> {
    self.source.as_ref().map(|err| err.downcast_ref()).flatten()
  }

  #[doc(hidden)]
  #[must_use]
  pub fn with_path(self, path: Vec<PathSegment>) -> Self {
    Self { path, ..self }
  }
}

impl Display for ServerError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.message)
  }
}

impl From<ServerError> for Vec<ServerError> {
  fn from(single: ServerError) -> Self {
    vec![single]
  }
}

impl From<parser::Error> for ServerError {
  fn from(e: parser::Error) -> Self {
    Self {
      message: e.to_string(),
      source: None,
      locations: e.positions().collect(),
      path: Vec::new(),
      extensions: None,
    }
  }
}

/// A segment of path to a resolver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PathSegment {
  /// A field in an object.
  Field(String),
  /// An index in a list.
  Index(usize),
}

/// Alias for Result<T, ServerError>
pub type ServerResult<T> = std::result::Result<T, ServerError>;

/// An error parsing an input value.
///
/// This type is generic over T as it uses T's type name when converting to a
/// regular error
pub struct InputValueError<T> {
  message: String,
  extensions: Option<ErrorExtensionValues>,
  phantom: PhantomData<T>,
}

impl<T: InputType> InputValueError<T> {
  fn new(message: String, extensions: Option<ErrorExtensionValues>) -> Self {
    Self {
      message,
      extensions,
      phantom: PhantomData,
    }
  }

  /// The expected input type didn't match the actual input type.
  #[must_use]
  pub fn expected_type(actual: Value) -> Self {
    Self::new(
      format!(
        r#"Expected input type "{}", found "{}" "#,
        T::type_name(),
        actual
      ),
      None,
    )
  }

  /// A custom error message.
  ///
  /// Any type that implements `Display` is automatically converted to this if
  /// you use `?` operator.
  pub fn custom(msg: impl Display) -> Self {
    Self::new(
      format!(r#"Failed to parse "{}": {}"#, T::type_name(), msg),
      None,
    )
  }

  /// Propogate the error message to a different type.
  pub fn propogate<U: InputType>(self) -> InputValueError<U> {
    if T::type_name() != U::type_name() {
      InputValueError::new(
        format!(
          r#"{} (occured while parsing"{}")"#,
          self.message,
          U::type_name(),
        ),
        self.extensions,
      )
    } else {
      InputValueError::new(self.message, self.extensions)
    }
  }

  pub fn with_extension(&mut self, name: impl AsRef<str>, value: impl Into<Value>) {
    self
      .extensions
      .get_or_insert_with(ErrorExtensionValues::default)
      .set(name, value);
  }

  /// Conver the error into a server error.
  pub fn into_server_error(self, pos: Pos) -> ServerError {
    let mut err = ServerError::new(self.message, Some(pos));
    err.extensions = self.extensions;
    err
  }
}

impl<T: InputType, E: Display> From<E> for InputValueError<T> {
  fn from(error: E) -> Self {
    Self::custom(error)
  }
}

/// An error parsing a value of type `T`.
pub type InputValueResult<T> = Result<T, InputValueError<T>>;

#[derive(Clone, Serialize)]
/// An error with a message and optional extensions.
pub struct Error {
  /// The error message.
  pub message: String,
  /// The source of the error.
  #[serde(skip)]
  pub source: Option<Arc<dyn Any + Send + Sync>>,
  /// Extensions to the error.
  #[serde(skip_serializing_if = "error_extensions_is_empty")]
  pub extensions: Option<ErrorExtensionValues>,
}

impl Debug for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Error")
      .field("message", &self.message)
      .field("extensions", &self.extensions)
      .finish()
  }
}

impl PartialEq for Error {
  fn eq(&self, other: &Self) -> bool {
    self.message.eq(&other.message) && self.extensions.eq(&other.extensions)
  }
}

impl Error {
  /// Create an error from the given error message.
  pub fn new(message: impl Into<String>) -> Self {
    Self {
      message: message.into(),
      source: None,
      extensions: None,
    }
  }

  /// Convert the error into a server error.
  #[must_use]
  pub fn into_server_error(self, pos: Pos) -> ServerError {
    ServerError {
      message: self.message,
      source: self.source,
      locations: vec![pos],
      path: Vec::new(),
      extensions: self.extensions,
    }
  }
}

/// An alias for `Result<T, Error>`
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseRequestError {
  /// An IO error occurred.
  #[error("{0}")]
  Io(#[from] std::io::Error),

  /// This request is a batch request, but the server doesn't support
  /// batch requests.
  #[error("Batch request are not supported.")]
  UnsupportedBatch,
}
