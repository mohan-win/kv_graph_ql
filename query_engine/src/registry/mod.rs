//! Impements necessary meta-data types for introspection.
mod cache_control;
mod meta_types;
pub use cache_control::CacheControl;
use chrono::{DateTime, Utc};
pub use meta_types::*;

use crate::graphql_parser::types::{
  BaseType as ParsedBaseType, ServiceDocument, Type as ParsedType, TypeSystemDefinition,
  VariableDefinition,
};
use crate::InputType;
use crate::{
  graphql_value::ConstValue as Value, introspection::types::__DirectiveLocation,
  schema::IntrospectionMode,
};
use core::panic;
use indexmap::{map::IndexMap, set::IndexSet};
use std::any::{type_name, TypeId};
use std::{
  collections::{BTreeMap, HashMap, HashSet},
  fmt,
  sync::Arc,
};

#[derive(Default, Debug)]
struct InterfacesImplementedByMap {
  /// Map<Key = InterfaceName, Value = Set(ObjectTypeNamesImplementingThisInterface)>
  by_obj_types: HashMap<String, IndexSet<String>>,
  /// Map<Key = InterfaceName, Value = Set(InterfacesImplementingThisInterface)>
  by_ifaces: HashMap<String, IndexSet<String>>,
}

impl InterfacesImplementedByMap {
  fn possible_types(&self, ty_name: &str) -> &IndexSet<String> {
    &self.by_obj_types[ty_name]
  }

  /// This method finds and updates the `by_obj_types` map with
  /// indirect sub-object-types of the interfaces recorded in `by_ifaces` map.
  fn update_indirect_sub_types(&mut self) {
    for (base_iface, implementing_ifaces) in self.by_ifaces.iter() {
      let indirect_types = implementing_ifaces.iter().fold(
        IndexSet::new(),
        |mut acc, implementing_iface| {
          self
            .by_obj_types
            .get(implementing_iface)
            .map(|obj_types| acc.extend(obj_types.iter().cloned()));
          acc
        },
      );
      let implemented_by = self.by_obj_types.entry(base_iface.clone()).or_default();
      implemented_by.extend(indirect_types.into_iter());
    }
  }

  fn update(
    &mut self,
    ty_name: &str,
    type_id: MetaTypeId,
    implements: &IndexSet<String>,
  ) {
    assert!(
      matches!(type_id, MetaTypeId::Object) || matches!(type_id, MetaTypeId::Interface),
    );

    implements.iter().for_each(|iface| match type_id {
      MetaTypeId::Object => {
        self
          .by_obj_types
          .entry(iface.clone())
          .and_modify(|implemented_by| {
            implemented_by.insert(ty_name.to_string());
          })
          .or_insert({
            let mut implemented_by = IndexSet::default();
            implemented_by.insert(ty_name.to_string());
            implemented_by
          });
      }
      MetaTypeId::Interface => {
        self
          .by_ifaces
          .entry(iface.clone())
          .and_modify(|implemented_by| {
            implemented_by.insert(ty_name.to_string());
          })
          .or_insert({
            let mut implemented_by = IndexSet::default();
            implemented_by.insert(ty_name.to_string());
            implemented_by
          });
      }
      _ => panic!("Only objects or interfaces can implement other interfaces"),
    });
  }
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

  implemented_by_map: InterfacesImplementedByMap,
}

impl Registry {
  /// Builds the registry for the given ServiceDocument.
  pub fn build_registry(service_doc: ServiceDocument) -> Self {
    let mut registry = Registry::default();
    // Note: Since schema_traspiler::graphql_gen uses default root
    // operation type names, We don't need to bother about TypeSystemDefinition::Schema.
    service_doc
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
    registry.implemented_by_map.update_indirect_sub_types();
    registry.query_type = registry
      .types
      .get("Query")
      .map(|_| "Query".to_string())
      .expect("There should be a root query type named `Query`");
    registry.mutation_type = registry
      .types
      .get("Mutation")
      .map(|_| "Mutation".to_string());
    registry.subscription_type = None;
    registry.introspection_mode = IntrospectionMode::default();
    registry.add_system_types(); // Add system types.

    registry
  }

  /// Checks if the `abstract_ty` is implemented by the type identified by the `implemented_by_ty_name`.
  #[inline]
  pub fn is_possible_type(
    &self,
    abstract_ty: &MetaType,
    implemented_by_ty_name: &str,
  ) -> bool {
    match abstract_ty {
      MetaType::Interface { name, .. } | MetaType::Union { name, .. } => self
        .get_possible_types(&name)
        .map_or(false, |possible_types| {
          possible_types.contains(implemented_by_ty_name)
        }),
      MetaType::Object { name, .. } => name.eq(implemented_by_ty_name),
      _ => false,
    }
  }

  /// If the type of given name is an abstract type (i.e, Union, Interface),
  /// Then this function returns possible concrete types for the given type name.
  #[inline]
  pub fn get_possible_types(&self, type_name: &str) -> Option<&IndexSet<String>> {
    self.types.get(type_name).map_or(None, |ty| match ty {
      MetaType::Union { possible_types, .. } => Some(&possible_types),
      MetaType::Interface { .. } => {
        Some(self.implemented_by_map.possible_types(type_name))
      }
      _ => None,
    })
  }

  /// Checks if the given MetaTypes overlap.
  /// * Arguments
  /// - ty1 - Meta type one
  /// - ty2 - Meta type two
  /// * Returns
  /// - `true` if the given types overlap.
  pub fn type_overlap(&self, ty1: &MetaType, ty2: &MetaType) -> bool {
    if std::ptr::eq(ty1, ty2) {
      return true;
    }
    match (ty1.is_abstract(), ty2.is_abstract()) {
      (true, true) => self
        .get_possible_types(ty1.name())
        .iter()
        .copied()
        .flatten()
        .any(|type_name| self.is_possible_type(ty2, type_name)),
      (true, false) => self.is_possible_type(ty1, ty2.name()),
      (false, true) => self.is_possible_type(ty2, ty1.name()),
      (false, false) => false,
    }
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
      name: "include".into(),
      description: Some("Directs the executor to include this field or fragment only when the `if` argument is true.".to_string()),
      locations: vec![
        __DirectiveLocation::FIELD,
        __DirectiveLocation::FRAGMENT_SPREAD,
        __DirectiveLocation::INLINE_FRAGMENT,
      ],
      args: {
        let mut args = IndexMap::new();
        args.insert("if".to_string(), MetaInputValue {
          name: "if".to_string(),
          description: Some("Included when true.".to_string()),
          ty: "Boolean!".to_string(),
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
    self.add_type(<bool as InputType>::create_type_info());
    self.add_type(<i32 as InputType>::create_type_info());
    self.add_type(<f64 as InputType>::create_type_info());
    self.add_type(<String as InputType>::create_type_info());
    self.add_type(crate::scalar::id::ID::create_type_info());
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
          MetaType::Interface {
            name, implements, ..
          }
          | MetaType::Object {
            name, implements, ..
          } => {
            self
              .implemented_by_map
              .update(&name, r#type.type_id(), implements);
          }
          _ => {
            // Do nothing.
          }
        }
        // Check for known custom scalars.
        match r#type {
          MetaType::Scalar { name, .. } if name.eq("DateTime") => {
            self
              .types
              .insert(name.to_string(), DateTime::<Utc>::create_type_info());
          }
          _ => {
            self.types.insert(name.to_string(), r#type);
          }
        }
      }
    }
  }

  fn add_directive(&mut self, directive: MetaDirective) {
    self
      .directives
      .insert(directive.name.to_string(), directive);
  }
}
