//! GraphQL code generation.
//!
//! This module exposes necessary functions to generate GraphQL types for
//! SDML models.
//!
mod error;
mod open_crud;
mod where_input;

use super::*;
pub use error::ErrorGraphQLGen;
use graphql_ast::*;
use open_crud::*;

pub const ID_FIELD_NAME: &str = "id";
pub type GraphQLGenResult<T> = Result<T, ErrorGraphQLGen>;

/// Date time scalar definition.
fn scalar_date_time_def() -> TypeDefinition {
    TypeDefinition {
      extend: false,
      description: Some("A date-time string at UTC, such as 2007-12-03T10:15:30Z, compliant with the date-timeformat outlined in section 5.6 of the RFC 3339 profile of the ISO 8601 standard for representationof dates and times using the Gregorian calendar.".to_string()),
      name: Name::new("DateTime"),
      directives: vec![],
      kind: TypeKind::Scalar,
    }
}

/// @map directive definition.
fn directive_map_def() -> DirectiveDefinition {
    DirectiveDefinition {
        description: Some(
            "This object field maps to a different field name in SDML model.".to_string(),
        ),
        name: Name::new("map"),
        arguments: vec![InputValueDefinition {
            description: Some("SDML model field name".to_string()),
            name: Name::new("name"),
            ty: Type::new("String!").unwrap(),
            default_value: None,
            directives: vec![],
        }],
        is_repeatable: false,
        locations: vec![DirectiveLocation::FieldDefinition],
    }
}

/// @unique directive definition.
fn directive_unique_def() -> DirectiveDefinition {
    DirectiveDefinition {
        description: Some("When applied to an object field, the value of the field should be unique across all object instances of the same type".to_string()),
        name: Name::new("unique"),
        arguments: vec![],
        is_repeatable: false,
        locations: vec![DirectiveLocation::FieldDefinition],
    }
}

/// Node interface definition.
fn interface_node_def() -> TypeDefinition {
    TypeDefinition {
        extend: false,
        description: Some(
            "Node interface as per Relay GraphQL Global Object Identification Spec. https://relay.dev/docs/guides/graphql-server-specification/#object-identification".to_string(),
        ),
        name: Name::new("Node"),
        directives: vec![],
        kind: TypeKind::Interface(InterfaceType {
            implements: vec![],
            fields: vec![FieldDefinition {
                description: Some("ID field with globally unique ID".to_string()),
                name: Name::new("id"),
                arguments: vec![],
                ty: Type::new("ID!").unwrap(),
                directives: vec![ConstDirective {
                    name: Name::new("unique"),
                    arguments: vec![],
                }],
            }],
        }),
    }
}

/**
 * Generate filters for individual types
 * - WhereInput, WhereUniqueInput   
 * - OrderByInput
 * - Generate GraphQL Type for the model
 * - Create & Update types
 * - Root fields in Query & Mutation.
 */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_defs() {
        let expected_graph_ql = r#"
"""A date-time string at UTC, such as 2007-12-03T10:15:30Z, compliant with the date-timeformat outlined in section 5.6 of the RFC 3339 profile of the ISO 8601 standard for representationof dates and times using the Gregorian calendar."""
scalar DateTime
"#;
        let date_time_scalar = scalar_date_time_def();
        assert_eq!(expected_graph_ql, date_time_scalar.to_string())
    }

    #[test]
    fn test_directive_map_def() {
        let expected_graph_ql = r#"
"""This object field maps to a different field name in SDML model."""
directive @map(
"""SDML model field name"""
name: String!
) on
| FIELD_DEFINITION
"#;
        let map_directive = directive_map_def();
        assert_eq!(expected_graph_ql, map_directive.to_string());
    }

    #[test]
    fn test_directive_unique_def() {
        let expected_graph_ql = r#"
"""When applied to an object field, the value of the field should be unique across all object instances of the same type"""
directive @unique on
| FIELD_DEFINITION
"#;
        let unique_directive = directive_unique_def();
        assert_eq!(expected_graph_ql, unique_directive.to_string());
    }

    #[test]
    fn test_node_interface_def() {
        let expected_graph_ql = r#"
"""Node interface as per Relay GraphQL Global Object Identification Spec. https://relay.dev/docs/guides/graphql-server-specification/#object-identification"""
interface Node {
"""ID field with globally unique ID"""
id: ID! @unique
}
"#;
        let node_interface_def = interface_node_def();
        assert_eq!(expected_graph_ql, node_interface_def.to_string());
    }
}
