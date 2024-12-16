use super::ScalarType;
use crate::{graphql_value::ConstValue as Value, InputValueError, InputValueResult};
use serde_json::Number;

impl ScalarType for f64 {
  type RawScalarType = Self;

  fn description() -> Option<String> {
    Some("The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754].".to_string())
  }

  fn specified_by_url() -> Option<String> {
    Some("https://en.wikipedia.org/wiki/IEEE_floating_point".to_string())
  }

  fn type_name() -> std::borrow::Cow<'static, str> {
    std::borrow::Cow::Borrowed("Float")
  }

  fn parse(value: graphql_value::ConstValue) -> InputValueResult<Self> {
    match value {
      Value::Number(n) => Ok(
        n.as_f64()
          .ok_or_else(|| InputValueError::from("Invalid number"))?,
      ),
      _ => Err(InputValueError::expected_type(value)),
    }
  }

  fn is_valid(value: &Value) -> bool {
    matches!(value, Value::Number(_))
  }

  fn to_value(&self) -> Value {
    match Number::from_f64(*self) {
      Some(n) => Value::Number(n),
      None => Value::Null,
    }
  }

  fn as_raw_scalar(&self) -> &Self::RawScalarType {
    self
  }
}
