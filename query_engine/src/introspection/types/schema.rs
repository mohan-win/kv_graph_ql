use std::collections::HashSet;

use crate::registry;

use super::{directive::__Directive, r#type::__Type};

/// A GraphQL schema defines capabilities of a GraphQL server. It exposes
/// all available types and directives on the server, as well as entry points
/// for a query, mutation, and subscription operations.
pub struct __Schema<'a> {
    registry: &'a registry::Registry,
}

impl<'a> __Schema<'a> {
    pub fn new(registry: &'a registry::Registry) -> Self {
        __Schema { registry }
    }
}

impl<'a> __Schema<'a> {
    /// Description of __Schema for newer graphiql interospection schema
    /// requirements
    fn description(&self) -> String {
        String::from("A GraphQL Schema defines the capabilities of a GraphQL server. It exposes all available types and directives on the server, as well as the entry points for query, mutation, and subscription operations.")
    }

    /// A list of all types supported by this server.
    fn types(&self) -> Vec<__Type<'a>> {
        let mut types: Vec<_> = self
            .registry
            .types
            .values()
            .map(|ty| (ty.name(), __Type::new_simple(self.registry, ty)))
            .collect();

        types.sort_by(|a, b| a.0.cmp(b.0));
        types.into_iter().map(|(_, ty)| ty).collect()
    }

    /// The root query type.
    #[inline]
    fn query_type(&self) -> __Type<'a> {
        __Type::new_simple(
            self.registry,
            &self.registry.types[&self.registry.query_type],
        )
    }

    /// The root mutation type if this server supports.
    fn mutation_type(&self) -> Option<__Type<'a>> {
        self.registry
            .mutation_type
            .as_ref()
            .map(|ty| __Type::new_simple(self.registry, &self.registry.types[ty]))
    }

    /// The root subscription type if this server supports.
    fn subscription_type(&self) -> Option<__Type<'a>> {
        self.registry
            .subscription_type
            .as_ref()
            .map(|ty| __Type::new_simple(self.registry, &self.registry.types[ty]))
    }

    /// A list of all directives supported by this server.
    fn directives(&self) -> Vec<__Directive<'a>> {
        let mut directives: Vec<_> = self
            .registry
            .directives
            .values()
            .map(|directive| __Directive {
                registry: self.registry,
                directive,
            })
            .collect();
        directives.sort_by(|a, b| a.directive.name.cmp(&b.directive.name));
        directives
    }
}
