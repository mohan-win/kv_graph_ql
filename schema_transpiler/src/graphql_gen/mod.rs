//! GraphQL code generation.
//!
//! This module exposes necessary functions to generate GraphQL types for
//! SDML models.
//!
mod error;

use super::*;
pub use error::ErrorGraphQLGen;
use graphql_ast::*;

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

/// Unique directive definition.
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

/// Generates necessary filter arguments for a string field.
fn input_filters_str_field_def<'src>(
    field_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<Vec<InputValueDefinition>> {
    let field_name: &'src str = field_name
        .try_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    // Names of the fields whose type is a list
    let list_field_names_fmt = [("{}_in", "in list"), ("{}_not_in", "not in list")];
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

    let list_fields = list_field_names_fmt
        .into_iter()
        .map(|(field_format, field_desc)| {
            let field_name = Name::new(field_format.replace("{}", field_name));
            let field_type = Type::new("[String]").expect("This should be of type String list");
            InputValueDefinition {
                description: Some(field_desc.to_string()),
                name: field_name,
                ty: field_type,
                default_value: None,
                directives: vec![],
            }
        });

    let non_list_fields = non_list_field_names_fmt
        .into_iter()
        .map(|(field_format, field_desc)| {
            let field_name = Name::new(field_format.replace("{}", field_name));
            let field_type = Type::new("String").expect("This should be of type String");
            InputValueDefinition {
                description: Some(field_desc.to_string()),
                name: field_name,
                ty: field_type,
                default_value: None,
                directives: vec![],
            }
        });

    Ok(non_list_fields.chain(list_fields).collect())
}

enum NumberType {
    Integer,
    Float,
}
fn input_filters_number_field_def<'src>(
    field_name: &sdml_ast::Token<'src>,
    number_type: NumberType,
) -> GraphQLGenResult<Vec<InputValueDefinition>> {
    let field_name: &'src str = field_name
        .try_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    // Names of the fields whose type is a list
    let list_field_names_fmt = [("{}_in", "in list"), ("{}_not_in", "not in list")];
    let non_list_field_names_fmt = [
        ("{}", "equals"),
        ("{}_not", "not equals"),
        ("{}_lt", "less than"),
        ("{}_lte", "less than or equals"),
        ("{}_gt", "greater than"),
        ("{}_gte", "greater than or equals"),
    ];

    let (num_type, num_type_list) = match number_type {
        NumberType::Integer => ("Integer", "[Integer]"),
        NumberType::Float => ("Float", "[Float]"),
    };
    let list_fields = list_field_names_fmt
        .into_iter()
        .map(|(field_format, field_desc)| {
            let field_name = Name::new(field_format.replace("{}", field_name));
            let field_type = Type::new(num_type_list).expect("This should be of type String list");
            InputValueDefinition {
                description: Some(field_desc.to_string()),
                name: field_name,
                ty: field_type,
                default_value: None,
                directives: vec![],
            }
        });

    let non_list_fields = non_list_field_names_fmt
        .into_iter()
        .map(|(field_format, field_desc)| {
            let field_name = Name::new(field_format.replace("{}", field_name));
            let field_type = Type::new(num_type).expect("This should be of type String");
            InputValueDefinition {
                description: Some(field_desc.to_string()),
                name: field_name,
                ty: field_type,
                default_value: None,
                directives: vec![],
            }
        });

    Ok(non_list_fields.chain(list_fields).collect())
}

/*fn model_where_input_def<'src>(
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
}*/

#[cfg(test)]
mod tests {
    use sdml_ast::Span;

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
    fn test_directive_defs() {
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

    #[test]
    fn test_input_filters_str_field_def() {
        let expected_str = r#"
"""equals"""
field: String
"""not equals"""
field_not: String
"""contains substring"""
field_contains: String
"""doesn't contain substring"""
field_not_contains: String
field_starts_with: String
field_not_starts_with: String
field_ends_with: String
field_not_ends_with: String
"""less than"""
field_lt: String
"""less than or equals"""
field_lte: String
"""greater than"""
field_gt: String
"""greater than or equals"""
field_gte: String
"""in list"""
field_in: [String]
"""not in list"""
field_not_in: [String]"#;
        let str_field_input_filters =
            input_filters_str_field_def(&sdml_ast::Token::Ident("field", Span::new(0, 0)))
                .expect("It should be a valid output");
        let actual_str = str_field_input_filters
            .into_iter()
            .fold("".to_string(), |acc, x| format!("{}{}", acc, x.to_string()));
        assert_eq!(expected_str, actual_str);
    }

    #[test]
    fn test_input_filters_int_field_def() {
        let expected_str = r#"
"""equals"""
field: Integer
"""not equals"""
field_not: Integer
"""less than"""
field_lt: Integer
"""less than or equals"""
field_lte: Integer
"""greater than"""
field_gt: Integer
"""greater than or equals"""
field_gte: Integer
"""in list"""
field_in: [Integer]
"""not in list"""
field_not_in: [Integer]"#;
        let int_field_input_filters = input_filters_number_field_def(
            &sdml_ast::Token::Ident("field", Span::new(0, 0)),
            NumberType::Integer,
        )
        .expect("It should be a valid output");
        let actual_str = int_field_input_filters
            .into_iter()
            .fold("".to_string(), |acc, x| format!("{}{}", acc, x.to_string()));
        assert_eq!(expected_str, actual_str);
    }

    #[test]
    fn test_input_filters_float_field_def() {
        let expected_str = r#"
"""equals"""
field: Float
"""not equals"""
field_not: Float
"""less than"""
field_lt: Float
"""less than or equals"""
field_lte: Float
"""greater than"""
field_gt: Float
"""greater than or equals"""
field_gte: Float
"""in list"""
field_in: [Float]
"""not in list"""
field_not_in: [Float]"#;
        let float_field_input_filters = input_filters_number_field_def(
            &sdml_ast::Token::Ident("field", Span::new(0, 0)),
            NumberType::Float,
        )
        .expect("It should be a valid output");
        let actual_str = float_field_input_filters
            .into_iter()
            .fold("".to_string(), |acc, x| format!("{}{}", acc, x.to_string()));
        assert_eq!(expected_str, actual_str);
    }
}
