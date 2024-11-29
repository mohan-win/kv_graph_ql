pub mod r#bool;
pub mod datetime;
pub mod float;
pub mod id;
pub mod int;
pub mod string;

use std::{borrow::Cow, sync::Arc};

use crate::{registry::MetaType, InputType, InputValueResult, Value};

/// All supported GraphQL scalars will conform to this trait.
pub trait ScalarType: Sized + Sync + Send {
  /// Raw type of the scalar type. Usually it is `Self`
  type RawScalarType;

  fn type_name() -> Cow<'static, str>;

  fn specified_by_url() -> Option<String> {
    return None;
  }

  fn description() -> Option<String> {
    return None;
  }

  /// Parse a scalar value.
  fn parse(value: Value) -> InputValueResult<Self>;

  /// Checks for a valid scalar value.
  ///
  /// Implementing this function can find incorrect input values
  /// during the verification phase.
  fn is_valid(_value: &Value) -> bool {
    true
  }

  /// Convert the scalar to `Value`
  fn to_value(&self) -> Value;

  /// Reference to the raw value.
  fn as_raw_scalar(&self) -> &Self::RawScalarType;
}

impl<T> InputType for T
where
  T: ScalarType,
{
  type RawValueType = T::RawScalarType;

  fn create_type_info() -> MetaType {
    MetaType::Scalar {
      name: <Self as ScalarType>::type_name().to_string(),
      description: <Self as ScalarType>::description(),
      is_valid: Option::Some(Arc::new(|value| <Self as ScalarType>::is_valid(value))),
      specified_by_url: <Self as ScalarType>::specified_by_url(),
    }
  }

  fn type_name() -> Cow<'static, str> {
    <Self as ScalarType>::type_name()
  }

  fn parse(value: Option<Value>) -> InputValueResult<Self> {
    <Self as ScalarType>::parse(value.unwrap_or_default())
  }

  fn to_value(&self) -> Value {
    <Self as ScalarType>::to_value(self)
  }

  fn as_raw_value(&self) -> Option<&Self::RawValueType> {
    Some(<Self as ScalarType>::as_raw_scalar(self))
  }
}
