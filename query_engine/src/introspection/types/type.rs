use std::collections::HashSet;

use crate::registry;

enum TypeDetail<'a> {
    Named(&'a registry::MetaType),
    NonNull(String),
    List(String),
}

/// The fundamental unit of any GraphQL schema is type. There are many kinds
/// of types in GraphQL as represented by __TypeKind enum.
///
/// Depending on the kind of type, certain fields describe information about
/// that type. Scalar types provide no information beyond a name and
/// description, while enum type provide their values. Object and interface
/// provide the object types possible at runtime. List and NonNull types compose
/// other types.
pub struct __Type<'a> {
    registry: &'a registry::Registry,
    visible_types: &'a HashSet<&'a str>,
    detail: TypeDetail<'a>,
}

impl<'a> __Type<'a> {
    #[inline]
    pub fn new_simple(
        registry: &'a registry::Registry,
        visible_types: &'a HashSet<&'a str>,
        ty: &'a registry::MetaType,
    ) -> Self {
        __Type {
            registry,
            visible_types,
            detail: TypeDetail::Named(ty),
        }
    }

    pub fn new(
        registry: &'a registry::Registry,
        visible_types: &'a HashSet<&'a str>,
        type_name: &'a str,
    ) -> Self {
        match registry::MetaTypeName::create(type_name) {
            registry::MetaTypeName::NonNull(ty) => __Type {
                registry,
                visible_types,
                detail: TypeDetail::NonNull(ty.to_string()),
            },
            registry::MetaTypeName::List(ty) => __Type {
                registry,
                visible_types,
                detail: TypeDetail::List(ty.to_string()),
            },
            registry::MetaTypeName::Named(ty) => __Type {
                registry,
                visible_types,
                detail: TypeDetail::Named(match registry.types.get(type_name) {
                    Some(ty) => ty,
                    None => panic!("Type {} not found!", ty),
                }),
            },
        }
    }
}
