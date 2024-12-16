use super::ScalarType;
use crate::{graphql_value::ConstValue as Value, InputValueError, InputValueResult};

impl ScalarType for bool {
  type RawScalarType = Self;

  fn description() -> Option<String> {
    Some("Built-in scalar type for Boolean values.".to_string())
  }

  fn type_name() -> std::borrow::Cow<'static, str> {
    std::borrow::Cow::Borrowed("Boolean")
  }
  fn parse(value: Value) -> InputValueResult<Self> {
    match value {
      Value::Boolean(n) => Ok(n),
      _ => Err(InputValueError::expected_type(value)),
    }
  }

  fn is_valid(value: &Value) -> bool {
    matches!(value, Value::Boolean(_))
  }

  fn to_value(&self) -> Value {
    Value::Boolean(*self)
  }

  fn as_raw_scalar(&self) -> &Self::RawScalarType {
    self
  }
}
