use super::ScalarType;
use crate::{InputValueError, InputValueResult, Value};

impl ScalarType for String {
  type RawScalarType = Self;

  fn description() -> Option<String> {
    Some(
      r#"The `String` scalar type represents textual data, represented as UTF-8
character sequences. The String type is most often used by GraphQL to
represent free-form human-readable text."#
        .to_string(),
    )
  }

  fn type_name() -> std::borrow::Cow<'static, str> {
    std::borrow::Cow::Borrowed("String")
  }

  fn parse(value: Value) -> InputValueResult<Self> {
    match value {
      Value::String(s) => Ok(s),
      _ => Err(InputValueError::expected_type(value)),
    }
  }

  fn is_valid(value: &Value) -> bool {
    matches!(value, Value::String(_))
  }

  fn to_value(&self) -> Value {
    Value::String(self.clone())
  }

  fn as_raw_scalar(&self) -> &Self::RawScalarType {
    self
  }
}
