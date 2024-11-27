//! Impements necessary meta-data types for introspection.
mod meta_types;
pub use meta_types::*;

use crate::graphql_parser::types::{
  BaseType as ParsedBaseType, ServiceDocument, Type as ParsedType, TypeSystemDefinition,
  VariableDefinition,
};
use crate::{
  introspection::types::__DirectiveLocation, schema::IntrospectionMode, Value,
};
use core::panic;
use indexmap::{map::IndexMap, set::IndexSet};
use std::any::TypeId;
use std::{
  collections::{BTreeMap, HashMap, HashSet},
  fmt,
  sync::Arc,
};

#[derive(Default)]
pub struct ImplementedBy {
  objects: IndexSet<String>,
  interfaces: IndexSet<String>,
}

/// A type registry for schema.
#[derive(Default)]
pub struct Registry {
  pub types: BTreeMap<String, MetaType>,
  pub directives: BTreeMap<String, MetaDirective>,
  pub query_type: String,
  pub mutation_type: Option<String>,
  pub subscription_type: Option<String>,
  pub introspection_mode: IntrospectionMode,
  pub ignore_name_conflicts: HashSet<String>,
  pub enable_suggestions: bool,

  /// implementation map: Map<Key = Interface Type Name, Value = ImplementedBy>
  implementation_map: HashMap<String, ImplementedBy>,
}

impl Registry {
  /// Builds the registry for the given ServiceDocument.
  pub fn build_registry(schema_doc: ServiceDocument) -> Self {
    let mut registry = Registry::default();
    // Note: Since schema_traspiler::graphql_gen uses default root
    // operation type names, We don't need to bother about TypeSystemDefinition::Schema.
    schema_doc
      .definitions
      .into_iter()
      .for_each(|def| match def {
        TypeSystemDefinition::Schema(_) => {
          panic!("The root operation types should have default name.")
        }
        TypeSystemDefinition::Directive(directive) => {
          registry.add_directive(directive.node.into())
        }
        TypeSystemDefinition::Type(ty) => registry.add_type(ty.node.into()),
      });
    registry.query_type = registry
      .types
      .get("Query")
      .map(|_| "Query".to_string())
      .expect("There should be a root query type named `Query`");
    registry.mutation_type = Some(
      registry
        .types
        .get("Mutation")
        .map(|_| "Mutation".to_string())
        .expect("There should be root mutation type named `Mutation`"),
    );
    registry.subscription_type = None;
    registry.introspection_mode = IntrospectionMode::default();
    registry.add_system_types(); // Add system types.

    registry
  }

  /// If the type of given name is an abstract type (i.e, Union, Interface).
  /// This function returns possible concrete types for the given type.
  pub fn possible_types(&self, type_name: &str) -> Option<&IndexSet<String>> {
    self.types.get(type_name).map_or(None, |ty| match ty {
      MetaType::Union { possible_types, .. } => Some(&possible_types),
      MetaType::Interface { .. } => Some(&self.implementation_map[type_name].objects),
      _ => None,
    })
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

  fn add_system_types(&mut self) {
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
        });

    self.add_directive(MetaDirective {
      name: "deprecated".into(),
      description: Some(
        "Marks an element of a GraphQL schema as no longer supported.".to_string(),
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
            description: Some(
              "A reason why it is deprecated, formatted using Markdown syntax".into(),
            ),
            ty: "String".into(),
            default_value: Some(r#""No longer supported.""#.into()),
            directive_invocations: vec![],
          },
        );
        args
      },
      is_repeatable: false,
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
        });

    self.add_directive(MetaDirective {
            name: "oneOf".into(),
            description: Some("Indicates that an Input Object is a OneOf Input Object(and thus requires exactly one of its field to be provided)".into()),
            locations: vec![__DirectiveLocation::INPUT_OBJECT],
            args: Default::default(),
            is_repeatable: false,
        });

    // Create system scalars.
    self.add_type(Self::boolean_scalar_type());
    self.add_type(Self::integer_scalar_type());
    self.add_type(Self::float_scalar_type());
    self.add_type(Self::id_scalar_type());
    self.add_type(Self::string_scalar_type());
  }

  fn add_type(&mut self, r#type: MetaType) {
    let (name, type_id) = (r#type.name(), r#type.type_id());
    match self.types.get(name) {
      Some(ty) => {
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
        // Update implementation map for Object & Interface types.
        match &r#type {
          MetaType::Object { implements, .. }
          | MetaType::Interface { implements, .. } => {
            self.update_implementation_map(r#type.name(), r#type.type_id(), implements);
          }
          _ => {
            // Do Nothing.
          }
        }
        self.types.insert(name.to_string(), r#type);
      }
    }
  }

  fn update_implementation_map(
    &mut self,
    ty_name: &str,
    type_id: MetaTypeId,
    interfaces: &IndexSet<String>,
  ) {
    interfaces.iter().for_each(|iface| {
      self
        .implementation_map
        .entry(iface.clone())
        .and_modify(|implemented_by| {
          if type_id == MetaTypeId::Object {
            implemented_by.objects.insert(ty_name.to_string());
          } else if type_id == MetaTypeId::Interface {
            implemented_by.interfaces.insert(ty_name.to_string());
          } else {
            panic!("Only Objects and Interfaces can implement other interfaces!")
          }
        });
    });
  }

  fn add_directive(&mut self, directive: MetaDirective) {
    self
      .directives
      .insert(directive.name.to_string(), directive);
  }

  fn boolean_scalar_type() -> MetaType {
    MetaType::Scalar {
      name: "Boolean".to_string(),
      description: Some("Built-in scalar type for Boolean values".to_string()),
      is_valid: None,
      specified_by_url: None,
    }
  }

  fn integer_scalar_type() -> MetaType {
    MetaType::Scalar {
      name: "Int".to_string(),
      description: Some("Built-in scalar type for Int values".to_string()),
      is_valid: None,
      specified_by_url: None,
    }
  }

  fn float_scalar_type() -> MetaType {
    MetaType::Scalar {
      name: "Float".to_string(),
      description: Some("Built-in scalar type for Int values".to_string()),
      is_valid: None,
      specified_by_url: None,
    }
  }

  fn id_scalar_type() -> MetaType {
    MetaType::Scalar {
      name: "ID".to_string(),
      description: Some("Built-in scalar type for ID values".to_string()),
      is_valid: None,
      specified_by_url: None,
    }
  }

  fn string_scalar_type() -> MetaType {
    MetaType::Scalar {
      name: "String".to_string(),
      description: Some("Built-in scalar type for String values".to_string()),
      is_valid: None,
      specified_by_url: None,
    }
  }
}
