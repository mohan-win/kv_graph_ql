use std::collections::HashSet;

use crate::registry;

/// A GraphQL schema defines capabilities of a GraphQL server. It exposes
/// all available types and directives on the server, as well as entry points
/// for a query, mutation, and subscription operations.
pub struct __Schema<'a> {
    registry: &'a registry::Registry,
    visible_types: &'a HashSet<&'a str>,
}

impl<'a> __Schema<'a> {
    pub fn new(
        registry: &'a registry::Registry,
        visible_types: &'a HashSet<&'a str>,
    ) -> Self {
        __Schema {
            registry,
            visible_types,
        }
    }
}