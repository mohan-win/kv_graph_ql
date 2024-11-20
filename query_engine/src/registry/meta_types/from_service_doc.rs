//! Implements From conversion trait to convert from relevant parser::type::service::* into
//! meta types.
use graphql_parser::types::{
    ConstDirective, EnumValueDefinition, FieldDefinition, InputValueDefinition,
};
use indexmap::IndexMap;

use super::{
    Deprecation, MetaDirectiveInvocation, MetaEnumValue, MetaField, MetaInputValue,
};

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
            deprecation: deprecation,
            directive_invocations: value
                .directives
                .into_iter()
                .map(|directive| directive.node.into())
                .collect(),
        }
    }
}
