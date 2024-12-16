use super::ScalarType;
use crate::{graphql_value::ConstValue as Value, InputValueError, InputValueResult};
use chrono::{DateTime, Utc};

impl ScalarType for DateTime<Utc> {
  type RawScalarType = Self;

  fn description() -> Option<String> {
    Some(
      "DateTime in UTC timezone. The input/output is a string in RFC3339 format."
        .to_string(),
    )
  }

  fn specified_by_url() -> Option<String> {
    Some("https://datatracker.ietf.org/doc/html/rfc3339".to_string())
  }

  fn type_name() -> std::borrow::Cow<'static, str> {
    std::borrow::Cow::Borrowed("DateTime")
  }

  fn parse(value: Value) -> InputValueResult<Self> {
    match value {
      Value::String(s) => Ok(s.parse::<DateTime<Utc>>()?),
      _ => Err(InputValueError::expected_type(value)),
    }
  }

  fn is_valid(value: &Value) -> bool {
    // Keeping the validation minimal to help the Validation phase to be as fast as possible.
    matches!(value, Value::String(_)) // if s.parse::<DateTime<Utc>>().is_ok())
  }

  fn to_value(&self) -> Value {
    Value::String(self.to_rfc3339())
  }

  fn as_raw_scalar(&self) -> &Self::RawScalarType {
    self
  }
}
