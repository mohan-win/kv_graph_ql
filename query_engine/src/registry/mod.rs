//! Impements necessary meta-data types for introspection.
mod meta_types;
pub use meta_types::*;

use crate::graphql_parser::types::{
    BaseType as ParsedBaseType, Type as ParsedType, VariableDefinition,
};
use crate::{
    introspection::types::__DirectiveLocation, schema::IntrospectionMode, Value,
};
use core::panic;
use indexmap::{map::IndexMap, set::IndexSet};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt,
    sync::Arc,
};

/// A type registry for schema.
#[derive(Default)]
pub struct Registry {
    pub types: BTreeMap<String, MetaType>,
    pub directives: BTreeMap<String, MetaDirective>,
    pub implements: HashMap<String, IndexSet<String>>,
    pub query_type: String,
    pub mutation_type: Option<String>,
    pub subscription_type: Option<String>,
    pub introspection_mode: IntrospectionMode,
    pub ignore_name_conflicts: HashSet<String>,
    pub enable_suggestions: bool,
}

impl Registry {
    pub(crate) fn add_system_types(&mut self) {
        self.add_directive(MetaDirective {
            name: "skip".into(),
            description: Some("Directs the executor to skip this field or fragment when the `if` argument is true".to_string()),
            locations: vec![
                __DirectiveLocation::FIELD,
                __DirectiveLocation::FRAGMENT_SPREAD,
                __DirectiveLocation::INLINE_FRAGMENT,
            ],
            args: {
                let mut args = IndexMap::new();
                args.insert("if".into(), MetaInputValue {
                    name: "if".into(),
                    description: Some("Skipped when true.".into()),
                    ty: "Boolean!".into(),
                    default_value: None,
                    directive_invocations: vec![],
                });
                args
            },
            is_repeatable: false,
            composable: None
        });

        self.add_directive(MetaDirective {
            name: "deprecated".into(),
            description: Some(
                "Marks an element of a GraphQL schema as no longer supported."
                    .to_string(),
            ),
            locations: vec![
                __DirectiveLocation::FIELD_DEFINITION,
                __DirectiveLocation::ARGUMENT_DEFINITION,
                __DirectiveLocation::INPUT_FIELD_DEFINITION,
                __DirectiveLocation::ENUM_VALUE,
            ],
            args: {
                let mut args = IndexMap::new();
                args.insert(
                    "reason".into(),
                    MetaInputValue {
                        name: "reason".into(),
                        description: Some("A reason why it is deprecated, formatted using Markdown syntax".into()),
                        ty: "String".into(),
                        default_value: Some(r#""No longer supported.""#.into()),
                        directive_invocations: vec![]
                    },
                );
                args
            },
            is_repeatable: false,
            composable: None,
        });

        self.add_directive(MetaDirective {
            name: "specifiedBy".into(),
            description: Some("Provides a scalar specification URL for specifying the behaviour of custom scalar types.".into()),
            locations: vec![__DirectiveLocation::SCALAR],
            args: {
                let mut args = IndexMap::new();
                args.insert("url".into(), MetaInputValue {
                    name: "url".into(),
                    description: Some("URL that specifies the behaviour of this scalar.".into()),
                    ty: "String!".into(),
                    default_value: None,
                    directive_invocations: vec![]
                });
                args
            },
            is_repeatable: false,
            composable: None
        });

        self.add_directive(MetaDirective {
            name: "oneOf".into(),
            description: Some("Indicates that an Input Object is a OneOf Input Object(and thus requires exactly one of its field to be provided)".into()),
            locations: vec![__DirectiveLocation::INPUT_OBJECT],
            args: Default::default(),
            is_repeatable: false,
            composable: None,
        });

        // Create system scalars.
        self.create_type(
            &mut Self::boolean_scalar_type,
            "Boolean",
            MetaTypeId::Scalar,
        );
        self.create_type(&mut Self::integer_scalar_type, "Int", MetaTypeId::Scalar);
        self.create_type(&mut Self::float_scalar_type, "Float", MetaTypeId::Scalar);
        self.create_type(&mut Self::id_scalar_type, "ID", MetaTypeId::Scalar);
        self.create_type(&mut Self::string_scalar_type, "String", MetaTypeId::Scalar);
    }

    fn create_type<F: FnMut(&mut Registry) -> MetaType>(
        &mut self,
        f: &mut F,
        name: &str,
        type_id: MetaTypeId,
    ) {
        const FAKE_TYPE_NAME: &'static str = "__fake_type__";
        match self.types.get(name) {
            Some(ty) => {
                if FAKE_TYPE_NAME == ty.name() {
                    return;
                }

                if !self.ignore_name_conflicts.contains(name) {
                    panic!(
                        "Already type `{}` already registered as `{}`",
                        ty.name(),
                        ty.type_id()
                    );
                }

                if ty.type_id() != type_id {
                    panic!(
                        "Register `{}` as `{}`, but already registered as `{}`",
                        name,
                        type_id,
                        ty.type_id()
                    );
                }
            }
            None => {
                // Inserting a fake type before calling the function allowes recursive types
                // to exist.
                self.types
                    .insert(name.to_string(), type_id.create_fake_type(FAKE_TYPE_NAME));
                let ty = f(self);
                *self.types.get_mut(name).unwrap() = ty;
            }
        }
    }

    pub fn add_directive(&mut self, directive: MetaDirective) {
        self.directives
            .insert(directive.name.to_string(), directive);
    }

    pub fn add_implements(&mut self, ty: &str, interface: &str) {
        self.implements
            .entry(ty.to_string())
            .and_modify(|interfaces| {
                interfaces.insert(interface.to_string());
            })
            .or_insert({
                let mut interfaces = IndexSet::new();
                interfaces.insert(interface.to_string());
                interfaces
            });
    }

    pub fn concrete_type_by_name(&self, type_name: &str) -> Option<&MetaType> {
        self.types.get(MetaTypeName::concrete_typename(type_name))
    }

    pub fn concrete_type_by_parsed_type(
        &self,
        query_type: &ParsedType,
    ) -> Option<&MetaType> {
        match &query_type.base {
            ParsedBaseType::Named(name) => self.types.get(name.as_str()),
            ParsedBaseType::List(ty) => self.concrete_type_by_parsed_type(ty),
        }
    }

    fn boolean_scalar_type(_registry: &mut Registry) -> MetaType {
        MetaType::Scalar {
            name: "Boolean".to_string(),
            description: Some("Built-in scalar type for Boolean values".to_string()),
            is_valid: None,
            specified_by_url: None,
        }
    }

    fn integer_scalar_type(_registry: &mut Registry) -> MetaType {
        MetaType::Scalar {
            name: "Int".to_string(),
            description: Some("Built-in scalar type for Int values".to_string()),
            is_valid: None,
            specified_by_url: None,
        }
    }

    fn float_scalar_type(_registry: &mut Registry) -> MetaType {
        MetaType::Scalar {
            name: "Float".to_string(),
            description: Some("Built-in scalar type for Int values".to_string()),
            is_valid: None,
            specified_by_url: None,
        }
    }

    fn id_scalar_type(_registry: &mut Registry) -> MetaType {
        MetaType::Scalar {
            name: "ID".to_string(),
            description: Some("Built-in scalar type for ID values".to_string()),
            is_valid: None,
            specified_by_url: None,
        }
    }

    fn string_scalar_type(_registry: &mut Registry) -> MetaType {
        MetaType::Scalar {
            name: "String".to_string(),
            description: Some("Built-in scalar type for String values".to_string()),
            is_valid: None,
            specified_by_url: None,
        }
    }
}
