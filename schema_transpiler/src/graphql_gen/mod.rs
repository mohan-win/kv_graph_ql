//! GraphQL code generation.
//!
//! This module exposes necessary functions to generate GraphQL types for
//! SDML models.
//!
mod error;
use std::fmt::format;

use super::*;
pub use error::ErrorGraphQLGen;
use graphql_ast::InputValueDefinition;
use sdml_ast::{Span, Token};
pub type CodeGenResult<T> = Result<T, ErrorGraphQLGen>;

/// Date time scalar definition.
fn scalar_def_date_time() -> graphql_ast::TypeDefinition {
    graphql_ast::TypeDefinition {
      extend: false,
      description: Some("A date-time string at UTC, such as 2007-12-03T10:15:30Z, compliant with the date-timeformat outlined in section 5.6 of the RFC 3339 profile of the ISO 8601 standard for representationof dates and times using the Gregorian calendar.".to_string()),
      name: graphql_ast::Name::new("DateTime"),
      directives: vec![],
      kind: graphql_ast::TypeKind::Scalar,
    }
}

/// Unique directive definition.
fn directive_def_unique() -> graphql_ast::DirectiveDefinition {
    graphql_ast::DirectiveDefinition {
        description: Some("@unique directive on an object field. When applied to an object field, the value of the field should be unique across all objects of the same type".to_string()),
        name: graphql_ast::Name::new("unique"),
        arguments: vec![],
        is_repeatable: false,
        locations: vec![graphql_ast::DirectiveLocation::FieldDefinition],
    }
}

/// Node interface definition.
fn interface_def_node() -> graphql_ast::TypeDefinition {
    graphql_ast::TypeDefinition {
        extend: false,
        description: Some(
            "Node interface as per Relay GraphQL Global Object Identification Spec".to_string(),
        ),
        name: graphql_ast::Name::new("Node"),
        directives: vec![],
        kind: graphql_ast::TypeKind::Interface(graphql_ast::InterfaceType {
            implements: vec![],
            fields: vec![],
        }),
    }
}

/// Generates necessary filter arguments for a string field.
fn input_filters_str_field_def<'src>(field_name: &sdml_ast::Token<'src>) -> CodeGenResult<Vec<InputValueDefinition>> {
    let field_name: &'src str = field_name.try_ident_name().map_err(|(error, pos)| ErrorGraphQLGen::SDMLError { error, pos})?;
    // Names of the fields whose type is a list
    let list_field_names_fmt = [
        ("{}_in", "in list"),
        ("{}_not_in", "not in list"),
    ];
    let non_list_field_names_fmt = [
        ("{}", "equals"),
        ("{}_not", "not equals"),
        ("{}_contains", "contains substring"),
        ("{}_not_contains", "doesn't contain substring"),
        ("{}_starts_with", ""),
        ("{}_not_starts_with", ""),
        ("{}_ends_with", ""),
        ("{}_not_ends_with", ""),
        ("{}_lt", "less than"),
        ("{}_lte", "less than or equals"),
        ("{}_gt", "greater than"),
        ("{}_gte", "greater than or equals"),
    ];

    let list_fields = list_field_names_fmt.into_iter().for_each(|(field_format, field_desc)| {
        let field_name = field_format.replace("{}", field_name);
        InputValueDefinition {
            
        }
    });

}


fn model_where_input_def<'src>(
    model: &sdml_ast::ModelDecl<'src>,
) -> CodeGenResult<graphql_ast::TypeDefinition> {
    let name: graphql_ast::Name = model
        .name
        .try_into()
        .map_err(|(error, pos)| ErrorGraphQLGen::SDMLError { error, pos })?;

    let input_object_definition = graphql_ast::InputObjectType {

    }

    Ok(graphql_ast::TypeDefinition {
        extend: false,
        description: Some("Identifies the model".to_string()),
        name: name,
        directives: vec![],
        kind: graphql_ast::TypeKind::InputObject(graphql_ast::InputObjectType { fields: () }),
    })
}
