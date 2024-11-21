//! Implements From conversion trait to convert from relevant parser::type::service::* into
//! meta types.
use graphql_parser::types::{
    ConstDirective, DirectiveDefinition, DirectiveLocation, EnumValueDefinition,
    FieldDefinition, InputValueDefinition, TypeDefinition, TypeKind, TypeSystemDefinition,
};
use indexmap::IndexMap;

use crate::introspection::types::__DirectiveLocation;

use super::{
    Deprecation, MetaDirective, MetaDirectiveInvocation, MetaEnumValue, MetaField,
    MetaInputValue, MetaType,
};

impl From<TypeDefinition> for MetaType {
    fn from(value: TypeDefinition) -> Self {
        match value.kind {
            TypeKind::Scalar => from_scalar_type(value),
            TypeKind::Object(_) => from_object_type(value),
            TypeKind::Interface(_) => from_interface_type(value),
            TypeKind::Union(_) => from_union_type(value),
            TypeKind::Enum(_) => from_enum_type(value),
            TypeKind::InputObject(_) => from_input_obj(value),
        }
    }
}

fn from_scalar_type(value: TypeDefinition) -> MetaType {
    if let TypeKind::Scalar = value.kind {
        MetaType::Scalar {
            name: value.name.node.to_string(),
            description: value.description.map(|desc| desc.node),
            is_valid: None, // ToDo:: Add scalar validator.
            specified_by_url: None,
        }
    } else {
        panic!("value is not of type Scalar")
    }
}

fn from_object_type(value: TypeDefinition) -> MetaType {
    if let TypeKind::Object(obj_ty) = value.kind {
        MetaType::Object {
            name: value.name.node.to_string(),
            description: value.description.map(|desc| desc.node),
            implements: obj_ty
                .implements
                .into_iter()
                .map(|interface| interface.node.to_string())
                .collect(),
            fields: obj_ty
                .fields
                .into_iter()
                .map(|field| (field.node.name.node.to_string(), field.node.into()))
                .collect(),
            is_subscription: false,
            directive_invocations: value
                .directives
                .into_iter()
                .map(|directive| directive.node.into())
                .collect(),
        }
    } else {
        panic!("Value is not of type object.")
    }
}

fn from_interface_type(value: TypeDefinition) -> MetaType {
    if let TypeKind::Interface(interface_ty) = value.kind {
        MetaType::Interface {
            name: value.name.node.to_string(),
            description: value.description.map(|desc| desc.node),
            implements: interface_ty
                .implements
                .into_iter()
                .map(|interface| interface.node.to_string())
                .collect(),
            fields: interface_ty
                .fields
                .into_iter()
                .map(|field| (field.node.name.node.to_string(), field.node.into()))
                .collect(),
            directive_invocations: value
                .directives
                .into_iter()
                .map(|directive| directive.node.into())
                .collect(),
        }
    } else {
        panic!("Value is not of type interface.")
    }
}

fn from_union_type(value: TypeDefinition) -> MetaType {
    if let TypeKind::Union(union_ty) = value.kind {
        MetaType::Union {
            name: value.name.node.to_string(),
            description: value.description.map(|desc| desc.node),
            possible_types: union_ty
                .members
                .into_iter()
                .map(|member| member.node.to_string())
                .collect(),
        }
    } else {
        panic!("Value is not of type union.")
    }
}

fn from_enum_type(value: TypeDefinition) -> MetaType {
    if let TypeKind::Enum(enum_ty) = value.kind {
        MetaType::Enum {
            name: value.name.node.to_string(),
            description: value.description.map(|desc| desc.node),
            enum_values: enum_ty
                .values
                .into_iter()
                .map(|enum_value| {
                    (
                        enum_value.node.value.node.to_string(),
                        enum_value.node.into(),
                    )
                })
                .collect(),
            directive_invocations: value
                .directives
                .into_iter()
                .map(|directive| directive.node.into())
                .collect(),
        }
    } else {
        panic!("Value is not of type enum.")
    }
}

fn from_input_obj(mut value: TypeDefinition) -> MetaType {
    if let TypeKind::InputObject(input_obj) = value.kind {
        let oneof_directive_pos = value
            .directives
            .iter()
            .position(|directive| directive.node.name.node.as_str() == "oneOf");
        let oneof = oneof_directive_pos.map_or(false, |pos| {
            value.directives.remove(pos); // Removing "oneOf" from directives array.
            true
        });
        MetaType::InputObject {
            name: value.name.node.to_string(),
            description: value.description.map(|desc| desc.node),
            input_fields: input_obj
                .fields
                .into_iter()
                .map(|input_field| {
                    (
                        input_field.node.name.node.to_string(),
                        input_field.node.into(),
                    )
                })
                .collect(),
            oneof,
            directive_invocations: value
                .directives
                .into_iter()
                .map(|directive| directive.node.into())
                .collect(),
        }
    } else {
        panic!("Value is not of type input object.")
    }
}

impl From<ConstDirective> for MetaDirectiveInvocation {
    fn from(value: ConstDirective) -> Self {
        MetaDirectiveInvocation {
            name: value.name.node.to_string(),
            args: {
                let mut args = IndexMap::new();
                value.arguments.into_iter().for_each(|(k, v)| {
                    args.insert(k.node.to_string(), v.node);
                });
                args
            },
        }
    }
}

impl TryFrom<ConstDirective> for Deprecation {
    type Error = String;
    fn try_from(value: ConstDirective) -> Result<Self, Self::Error> {
        if value.name.node.as_str() == "deprecated" {
            let reason = value.arguments.iter().find_map(|(name, value)| {
                if name.node.as_str() == "reason" {
                    Some(value.node.to_string())
                } else {
                    None
                }
            });
            Ok(Deprecation::Deprecated { reason })
        } else {
            Err("It is not a deprecated directive.".to_string())
        }
    }
}

impl From<InputValueDefinition> for MetaInputValue {
    fn from(value: InputValueDefinition) -> Self {
        MetaInputValue {
            name: value.name.node.to_string(),
            description: value.description.map(|desc| desc.node),
            ty: value.ty.node.to_string(),
            default_value: value.default_value.map(|value| value.node.to_string()),
            directive_invocations: value
                .directives
                .into_iter()
                .map(|directive| directive.node.into())
                .collect(),
        }
    }
}

impl From<FieldDefinition> for MetaField {
    fn from(mut value: FieldDefinition) -> Self {
        let deprecated_pos = value
            .directives
            .iter()
            .position(|directive| directive.node.name.node.as_str() == "deprecated");
        let deprecation = deprecated_pos.map_or(Deprecation::NoDeprecated, |pos| {
            value.directives.remove(pos).node.try_into().unwrap()
        });
        MetaField {
            name: value.name.node.to_string(),
            description: value.description.map(|desc| desc.node),
            args: value
                .arguments
                .into_iter()
                .map(|arg| (arg.node.name.to_string(), arg.node.into()))
                .collect(),
            ty: value.ty.node.to_string(),
            deprecation,
            directive_invocations: value
                .directives
                .into_iter()
                .map(|directive| directive.node.into())
                .collect(),
        }
    }
}

impl From<EnumValueDefinition> for MetaEnumValue {
    fn from(mut value: EnumValueDefinition) -> Self {
        let deprecated_pos = value
            .directives
            .iter()
            .position(|directive| directive.node.name.node.as_str() == "deprecated");
        let deprecation = deprecated_pos.map_or(Deprecation::NoDeprecated, |pos| {
            value.directives.remove(pos).node.try_into().unwrap()
        });
        MetaEnumValue {
            name: value.value.node.to_string(),
            description: value.description.map(|desc| desc.node),
            deprecation,
            directive_invocations: value
                .directives
                .into_iter()
                .map(|directive| directive.node.into())
                .collect(),
        }
    }
}

impl From<DirectiveLocation> for __DirectiveLocation {
    fn from(value: DirectiveLocation) -> Self {
        match value {
            DirectiveLocation::Query => Self::QUERY,

            DirectiveLocation::Mutation => Self::MUTATION,

            DirectiveLocation::Subscription => Self::SUBSCRIPTION,

            DirectiveLocation::Field => Self::FIELD,

            DirectiveLocation::FragmentDefinition => Self::FIELD_DEFINITION,

            DirectiveLocation::FragmentSpread => Self::FRAGMENT_SPREAD,

            DirectiveLocation::InlineFragment => Self::INLINE_FRAGMENT,

            DirectiveLocation::Schema => Self::SCHEMA,

            DirectiveLocation::Scalar => Self::SCALAR,

            DirectiveLocation::Object => Self::OBJECT,

            DirectiveLocation::FieldDefinition => Self::FIELD_DEFINITION,

            DirectiveLocation::ArgumentDefinition => Self::ARGUMENT_DEFINITION,

            DirectiveLocation::Interface => Self::INTERFACE,

            DirectiveLocation::Union => Self::UNION,

            DirectiveLocation::Enum => Self::ENUM,

            DirectiveLocation::EnumValue => Self::ENUM_VALUE,

            DirectiveLocation::InputObject => Self::INPUT_OBJECT,

            DirectiveLocation::InputFieldDefinition => Self::INPUT_FIELD_DEFINITION,

            DirectiveLocation::VariableDefinition => Self::VARIABLE_DEFINITION,
        }
    }
}

impl From<DirectiveDefinition> for MetaDirective {
    fn from(value: DirectiveDefinition) -> Self {
        MetaDirective {
            name: value.name.node.to_string(),
            description: value.description.map(|desc| desc.node),
            locations: value
                .locations
                .into_iter()
                .map(|loc| loc.node.into())
                .collect(),
            args: value
                .arguments
                .into_iter()
                .map(|arg| (arg.node.name.node.to_string(), arg.node.into()))
                .collect(),
            is_repeatable: value.is_repeatable,
        }
    }
}
