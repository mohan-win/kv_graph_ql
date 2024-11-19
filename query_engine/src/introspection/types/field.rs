use std::collections::HashSet;

use crate::registry;

use super::{input_value::__InputValue, r#type::__Type};

/// Object and Interface types are described by list of Fields, each of which
/// has a name, potentially a list of arguments and a return type.
pub struct __Field<'a> {
    pub registry: &'a registry::Registry,
    pub field: &'a registry::MetaField,
}

impl<'a> __Field<'a> {
    #[inline]
    fn name(&self) -> &str {
        &self.field.name
    }

    #[inline]
    fn description(&self) -> Option<&str> {
        self.field.description.as_deref()
    }

    fn args(&self) -> Vec<__InputValue> {
        // ToDo::
        // Avoiding visibility filter for the fields based on the context.
        // to be added later when it becomes necessary.
        self.field
            .args
            .values()
            .map(|input_value| __InputValue {
                registry: self.registry,
                input_value,
            })
            .collect()
    }

    #[inline]
    fn r#type(&self) -> __Type<'a> {
        __Type::new(self.registry, &self.field.ty)
    }

    #[inline]
    fn is_deprecated(&self) -> bool {
        self.field.deprecation.is_deprecated()
    }

    #[inline]
    fn deprecation_reason(&self) -> Option<&str> {
        self.field.deprecation.reason()
    }
}
