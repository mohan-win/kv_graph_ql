use std::collections::HashSet;

use crate::registry;

/// Object and Interface types are described by list of Fields, each of which
/// has a name, potentially a list of arguments and a return type.
pub struct __Field<'a> {
    pub registry: &'a registry::Registry,
    pub visible_types: &'a HashSet<&'a str>,
    pub field: &'a registry::MetaField,
}
