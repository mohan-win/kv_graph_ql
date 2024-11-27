use crate::registry;

/// One possible value for a given Enum. Enum values are unique values, not a
/// placeholder for a string or numeric. However a Enum is returned in a JSON response as string.
pub struct __EnumValue<'a> {
  pub value: &'a registry::MetaEnumValue,
}

impl<'a> __EnumValue<'a> {
  #[inline]
  fn name(&self) -> &str {
    &self.value.name
  }

  #[inline]
  fn description(&self) -> Option<&str> {
    self.value.description.as_deref()
  }

  #[inline]
  fn is_deprecated(&self) -> bool {
    self.value.deprecation.is_deprecated()
  }

  #[inline]
  fn deprecation_reason(&self) -> Option<&str> {
    self.value.deprecation.reason()
  }
}
