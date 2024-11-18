use std::collections::HashSet;

use crate::registry;

/// Arguments provided to Fields or Directives and the input fields of an InputObject
/// are represented as Input Values which describe their type and
/// optionally a default value.
pub struct __InputValue<'a> {
    pub registry: &'a registry::Registry,
    pub visible_types: &'a HashSet<&'a str>,
    pub input_value: &'a registry::MetaInputValue,
}
