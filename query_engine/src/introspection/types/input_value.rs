use std::collections::HashSet;

use crate::registry;

use super::r#type::__Type;

/// Arguments provided to Fields or Directives and the input fields of an InputObject
/// are represented as Input Values which describe their type and
/// optionally a default value.
pub struct __InputValue<'a> {
    pub registry: &'a registry::Registry,
    pub visible_types: &'a HashSet<&'a str>,
    pub input_value: &'a registry::MetaInputValue,
}

impl<'a> __InputValue<'a> {
    #[inline]
    fn name(&self) -> &str {
        &self.input_value.name
    }

    #[inline]
    fn description(&self) -> Option<&str> {
        self.input_value.description.as_deref()
    }

    #[inline]
    fn r#type(&self) -> __Type<'a> {
        __Type::new(self.registry, self.visible_types, &self.input_value.ty)
    }

    #[inline]
    fn default_value(&self) -> Option<&str> {
        self.input_value.default_value.as_deref()
    }
}
