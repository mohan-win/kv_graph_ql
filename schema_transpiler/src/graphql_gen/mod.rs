//! GraphQL code generation.
//!
//! This module exposes necessary functions to generate GraphQL types for
//! SDML models.
//!

use super::*;

pub fn scalar_date_time() -> graphql_ast::TypeDefinition {
    graphql_ast::TypeDefinition {
      extend: false,
      description: Some("A date-time string at UTC, such as 2007-12-03T10:15:30Z, compliant with the date-timeformat outlined in section 5.6 of the RFC 3339 profile of the ISO 8601 standard for representationof dates and times using the Gregorian calendar.".to_string()),
      name: graphql_ast::Name::new("DateTime"),
      directives: vec![],
      kind: graphql_ast::TypeKind::Scalar,
    }
}

pub fn unique_directive() -> graphql_ast::DirectiveDefinition {
    graphql_ast::DirectiveDefinition {
        description: Some("@unique directive on an object field. When applied to an object field, the value of the field should be unique across all objects of the same type".to_string()),
        name: graphql_ast::Name::new("unique"),
        arguments: vec![],
        is_repeatable: false,
        locations: vec![graphql_ast::DirectiveLocation::FieldDefinition],
    }
}
