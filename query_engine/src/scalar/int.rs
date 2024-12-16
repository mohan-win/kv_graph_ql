use super::ScalarType;
use crate::{graphql_value::ConstValue as Value, InputValueError, InputValueResult};
use serde_json::Number;

impl ScalarType for i32 {
  type RawScalarType = Self;

  fn description() -> Option<String> {
    Some(
      "The `Int` scalar type represents non-fractional whole numeric values.".to_string(),
    )
  }

  fn type_name() -> std::borrow::Cow<'static, str> {
    std::borrow::Cow::Borrowed("Int")
  }

  fn parse(value: graphql_value::ConstValue) -> InputValueResult<Self> {
    match value {
      Value::Number(n) => {
        let n = n
          .as_i64()
          .ok_or_else(|| InputValueError::from("Invalid number"))?;
        if n < Self::MIN as i64 || n > Self::MAX as i64 {
          return Err(InputValueError::from(format!(
            "Only integers from {} to {} are accepted",
            Self::MIN,
            Self::MAX
          )));
        }
        Ok(n as Self)
      }
      _ => Err(InputValueError::expected_type(value)),
    }
  }

  fn is_valid(value: &Value) -> bool {
    matches!(value, Value::Number(n) if n.is_i64())
  }

  fn to_value(&self) -> Value {
    Value::Number(Number::from(*self as i64))
  }

  fn as_raw_scalar(&self) -> &Self::RawScalarType {
    self
  }
}
