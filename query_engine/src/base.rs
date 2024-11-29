use std::borrow::Cow;

use crate::{registry::MetaType, InputValueResult, Value};

/// Represents a GraphQL input type.
pub trait InputType: Send + Sync + Sized {
  /// The raw type used for validator.
  ///
  /// Usually it is `Self`, but the wrapper type is its internal type.
  /// For example:
  /// `i32::RawValueType` is `i32`
  /// `Option<i32>::RawValueType` is `i32`.
  type RawValueType;

  fn create_type_info() -> MetaType;

  fn type_name() -> Cow<'static, str>;

  fn qualified_type_name() -> String {
    format!("{}!", Self::type_name())
  }

  /// Parse from `Value`. None represents undefined.
  fn parse(value: Option<Value>) -> InputValueResult<Self>;

  /// Convert to a `Value` for interospection.
  fn to_value(&self) -> Value;

  /// Returns reference to the raw value.
  fn as_raw_value(&self) -> Option<&Self::RawValueType>;
}
