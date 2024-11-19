use std::collections::HashSet;

use crate::registry::{self, MetaType};

use super::{
    enum_value::__EnumValue, field::__Field, input_value::__InputValue, kind::__TypeKind,
};

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
    detail: TypeDetail<'a>,
}

impl<'a> __Type<'a> {
    #[inline]
    pub fn new_simple(
        registry: &'a registry::Registry,
        ty: &'a registry::MetaType,
    ) -> Self {
        __Type {
            registry,
            detail: TypeDetail::Named(ty),
        }
    }

    pub fn new(registry: &'a registry::Registry, type_name: &'a str) -> Self {
        match registry::MetaTypeName::create(type_name) {
            registry::MetaTypeName::NonNull(ty) => __Type {
                registry,
                detail: TypeDetail::NonNull(ty.to_string()),
            },
            registry::MetaTypeName::List(ty) => __Type {
                registry,
                detail: TypeDetail::List(ty.to_string()),
            },
            registry::MetaTypeName::Named(ty) => __Type {
                registry,
                detail: TypeDetail::Named(match registry.types.get(type_name) {
                    Some(ty) => ty,
                    None => panic!("Type {} not found!", ty),
                }),
            },
        }
    }
}

impl<'a> __Type<'a> {
    #[inline]
    fn kind(&self) -> __TypeKind {
        match &self.detail {
            TypeDetail::Named(ty) => match ty {
                registry::MetaType::Scalar { .. } => __TypeKind::Scalar,
                registry::MetaType::Object { .. } => __TypeKind::Object,
                registry::MetaType::Enum { .. } => __TypeKind::Enum,
                registry::MetaType::Interface { .. } => __TypeKind::Interface,
                registry::MetaType::Union { .. } => __TypeKind::Union,
                registry::MetaType::InputObject { .. } => __TypeKind::InputObject,
            },
            TypeDetail::NonNull(_) => __TypeKind::NonNull,
            TypeDetail::List(_) => __TypeKind::List,
        }
    }

    #[inline]
    fn name(&self) -> Option<&str> {
        match &self.detail {
            TypeDetail::Named(ty) => Some(ty.name()),
            TypeDetail::NonNull(_) => None,
            TypeDetail::List(_) => None,
        }
    }

    #[inline]
    fn description(&self) -> Option<&str> {
        match self.detail {
            TypeDetail::Named(ty) => match ty {
                registry::MetaType::Scalar { description, .. }
                | registry::MetaType::Object { description, .. }
                | registry::MetaType::Interface { description, .. }
                | registry::MetaType::Enum { description, .. }
                | registry::MetaType::Union { description, .. }
                | registry::MetaType::InputObject { description, .. } => {
                    description.as_deref()
                }
            },
            TypeDetail::NonNull(_) => None,
            TypeDetail::List(_) => None,
        }
    }

    fn fields(&self, include_deprecated: bool) -> Option<Vec<__Field<'a>>> {
        if let TypeDetail::Named(ty) = &self.detail {
            ty.fields().map(|fields| {
                fields
                    .values()
                    .filter(|field| {
                        (include_deprecated || !field.deprecation.is_deprecated())
                            && !field.name.starts_with("__")
                    })
                    .map(|field| __Field {
                        registry: self.registry,
                        field,
                    })
                    .collect()
            })
        } else {
            None
        }
    }

    fn interfaces(&self) -> Option<Vec<__Type<'a>>> {
        if let TypeDetail::Named(registry::MetaType::Object { name, .. }) = &self.detail {
            self.registry.implements.get(name).map(|implements| {
                implements
                    .iter()
                    .map(|ty| __Type::new(self.registry, ty))
                    .collect()
            })
        } else {
            None
        }
    }

    fn possible_types(&self) -> Option<Vec<__Type<'a>>> {
        if let TypeDetail::Named(registry::MetaType::Interface {
            possible_types, ..
        })
        | TypeDetail::Named(registry::MetaType::Union { possible_types, .. }) =
            &self.detail
        {
            Some(
                possible_types
                    .iter()
                    .map(|ty| __Type::new(self.registry, ty))
                    .collect(),
            )
        } else {
            None
        }
    }

    fn enum_values(&self, include_deprecated: bool) -> Option<Vec<__EnumValue>> {
        if let TypeDetail::Named(registry::MetaType::Enum { enum_values, .. }) =
            &self.detail
        {
            Some(
                enum_values
                    .values()
                    .filter(|value| {
                        include_deprecated || !value.deprecation.is_deprecated()
                    })
                    .map(|value| __EnumValue { value })
                    .collect(),
            )
        } else {
            None
        }
    }

    fn input_fields(&self) -> Option<Vec<__InputValue<'a>>> {
        if let TypeDetail::Named(MetaType::InputObject { input_fields, .. }) =
            &self.detail
        {
            Some(
                input_fields
                    .values()
                    .map(|input_value| __InputValue {
                        registry: self.registry,
                        input_value,
                    })
                    .collect(),
            )
        } else {
            None
        }
    }

    fn of_type(&'a self) -> Option<__Type<'a>> {
        if let TypeDetail::List(ty) = &self.detail {
            Some(__Type::new(self.registry, &ty))
        } else if let TypeDetail::NonNull(ty) = &self.detail {
            Some(__Type::new(self.registry, &ty))
        } else {
            None
        }
    }

    fn specified_by_url(&self) -> Option<&'a str> {
        if let TypeDetail::Named(registry::MetaType::Scalar {
            specified_by_url, ..
        }) = &self.detail
        {
            specified_by_url.as_deref()
        } else {
            None
        }
    }

    fn is_one_of(&self) -> Option<bool> {
        if let TypeDetail::Named(registry::MetaType::InputObject { oneof, .. }) =
            &self.detail
        {
            Some(*oneof)
        } else {
            None
        }
    }
}
