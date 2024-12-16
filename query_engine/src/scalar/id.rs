use super::ScalarType;
use crate::{graphql_value::ConstValue as Value, InputValueError, InputValueResult};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(
  Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, Default,
)]
#[serde(transparent)]
pub struct ID(pub String);

impl AsRef<str> for ID {
  fn as_ref(&self) -> &str {
    self.0.as_str()
  }
}

impl Deref for ID {
  type Target = String;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for ID {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T: std::fmt::Display> From<T> for ID {
  fn from(value: T) -> Self {
    ID(value.to_string())
  }
}

impl From<ID> for String {
  fn from(id: ID) -> Self {
    id.0
  }
}

impl From<ID> for Value {
  fn from(id: ID) -> Self {
    Value::String(id.0)
  }
}

impl PartialEq<&str> for ID {
  fn eq(&self, other: &&str) -> bool {
    self.0.as_str() == *other
  }
}

impl ScalarType for ID {
  type RawScalarType = Self;

  fn description() -> Option<String> {
    Some("The `ID` scalar type represents unique identifier. The `ID` type serialized in the same way as `String`, but intended to be human readbale.".to_string())
  }

  fn type_name() -> std::borrow::Cow<'static, str> {
    return std::borrow::Cow::Borrowed("ID");
  }

  fn parse(value: Value) -> InputValueResult<Self> {
    match value {
      Value::Number(n) if n.is_i64() => Ok(ID(n.to_string())),
      Value::String(s) => Ok(ID(s)),
      _ => Err(InputValueError::expected_type(value)),
    }
  }

  fn is_valid(value: &Value) -> bool {
    match value {
      Value::Number(n) if n.is_i64() => true,
      Value::String(_) => true,
      _ => false,
    }
  }

  fn to_value(&self) -> Value {
    Value::String(self.0.clone())
  }

  fn as_raw_scalar(&self) -> &Self::RawScalarType {
    self
  }
}
