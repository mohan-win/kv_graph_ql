use super::*;

/// Get connection type and its edge, definition for given model.
pub fn connection_types_def<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<Vec<TypeDefinition>> {
    let mut result = vec![];
    let edge = edge_type_def(model_name)?;
    let edge_type_name = edge.name.as_str().to_string();
    result.push(edge);

    let model_name = model_name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    result.push(TypeDefinition {
        extend: false,
        description: None,
        name: AuxiliaryType::Connection.name(model_name),
        directives: vec![],
        kind: TypeKind::Object(ObjectType {
            implements: vec![],
            fields: vec![
                FieldDefinition {
                    description: None,
                    name: Name::new("pageInfo"),
                    arguments: vec![],
                    ty: open_crud::QueryType::PageInfo
                        .common_ty(sdml_ast::FieldTypeMod::NonOptional),
                    directives: vec![],
                },
                FieldDefinition {
                    description: None,
                    name: Name::new("edges"),
                    arguments: vec![],
                    ty: Type::new(&edge_type_name, sdml_ast::FieldTypeMod::Array),
                    directives: vec![],
                },
                FieldDefinition {
                    description: None,
                    name: Name::new("aggregate"),
                    arguments: vec![],
                    ty: open_crud::QueryType::Aggregate
                        .common_ty(sdml_ast::FieldTypeMod::NonOptional),
                    directives: vec![],
                },
            ],
        }),
    });

    Ok(result)
}

fn edge_type_def<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    let model_name = model_name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    Ok(TypeDefinition {
        extend: false,
        description: None,
        name: AuxiliaryType::Edge.name(model_name),
        directives: vec![],
        kind: TypeKind::Object(ObjectType {
            implements: vec![],
            fields: vec![
                FieldDefinition {
                    description: None,
                    name: Name::new("node"),
                    arguments: vec![],
                    ty: Type::new(model_name, sdml_ast::FieldTypeMod::NonOptional),
                    directives: vec![],
                },
                FieldDefinition {
                    description: None,
                    name: Name::new("cursor"),
                    arguments: vec![],
                    ty: Type::new(
                        FIELD_TYPE_NAME_STRING,
                        sdml_ast::FieldTypeMod::NonOptional,
                    ),
                    directives: vec![],
                },
            ],
        }),
    })
}

pub fn page_info_type_def<'src>() -> GraphQLGenResult<TypeDefinition> {
    Ok(TypeDefinition {
        extend: false,
        description: None,
        name: open_crud::QueryType::PageInfo.common_name(),
        directives: vec![],
        kind: TypeKind::Object(ObjectType {
            implements: vec![],
            fields: vec![
                FieldDefinition {
                    description: Some(
                        "When paginating forwards, are there more items ?".to_string(),
                    ),
                    name: Name::new("hasNextPage"),
                    arguments: vec![],
                    ty: Type::new(
                        FIELD_TYPE_NAME_BOOL,
                        sdml_ast::FieldTypeMod::NonOptional,
                    ),
                    directives: vec![],
                },
                FieldDefinition {
                    description: Some(
                        "When paginating backwards, are there more items ?".to_string(),
                    ),
                    name: Name::new("hasPreviousPage"),
                    arguments: vec![],
                    ty: Type::new(
                        FIELD_TYPE_NAME_BOOL,
                        sdml_ast::FieldTypeMod::NonOptional,
                    ),
                    directives: vec![],
                },
                FieldDefinition {
                    description: Some(
                        "When paginating backwards, cursor to continue.".to_string(),
                    ),
                    name: Name::new("startCursor"),
                    arguments: vec![],
                    ty: Type::new(
                        FIELD_TYPE_NAME_STRING,
                        sdml_ast::FieldTypeMod::Optional,
                    ),
                    directives: vec![],
                },
                FieldDefinition {
                    description: Some(
                        "When paginating forwards, cursor to continue.".to_string(),
                    ),
                    name: Name::new("endCursor"),
                    arguments: vec![],
                    ty: Type::new(
                        FIELD_TYPE_NAME_STRING,
                        sdml_ast::FieldTypeMod::Optional,
                    ),
                    directives: vec![],
                },
                FieldDefinition {
                    description: Some("Number of items in current page.".to_string()),
                    name: Name::new("pageSize"),
                    arguments: vec![],
                    ty: Type::new(FIELD_TYPE_NAME_INT, sdml_ast::FieldTypeMod::Optional),
                    directives: vec![],
                },
            ],
        }),
    })
}

pub fn aggregage_type_def<'src>() -> GraphQLGenResult<TypeDefinition> {
    Ok(TypeDefinition {
        extend: false,
        description: None,
        name: open_crud::QueryType::Aggregate.common_name(),
        directives: vec![],
        kind: TypeKind::Object(ObjectType {
            implements: vec![],
            fields: vec![FieldDefinition {
                description: None,
                name: Name::new("count"),
                arguments: vec![],
                ty: Type::new(FIELD_TYPE_NAME_INT, sdml_ast::FieldTypeMod::NonOptional),
                directives: vec![],
            }],
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdml_parser::ast::Span;

    #[test]
    fn test_aggregate_type_def() {
        let expected_graphql_str = r#"
type Aggregate {
count: Int!
}
"#;

        assert_eq!(
            expected_graphql_str,
            aggregage_type_def().unwrap().to_string()
        )
    }

    #[test]
    fn test_page_info_type_def() {
        let expected_graphql_str = r#"
type PageInfo {
"""When paginating forwards, are there more items ?"""
hasNextPage: Boolean!
"""When paginating backwards, are there more items ?"""
hasPreviousPage: Boolean!
"""When paginating backwards, cursor to continue."""
startCursor: String
"""When paginating forwards, cursor to continue."""
endCursor: String
"""Number of items in current page."""
pageSize: Int
}
"#;

        assert_eq!(
            expected_graphql_str,
            page_info_type_def().unwrap().to_string()
        )
    }
    #[test]
    fn test_edge_type_def() {
        let expected_graphql_str = r#"
type UserEdge {
node: User!
cursor: String!
}
"#;
        let user_edge_ty =
            edge_type_def(&sdml_parser::ast::Token::Ident("User", Span::new(0, 0)))
                .unwrap();
        assert_eq!(expected_graphql_str, user_edge_ty.to_string())
    }

    #[test]
    fn test_connection_type_def() {
        let expected_graphql_str = r#"
type UserEdge {
node: User!
cursor: String!
}

type UserConnection {
pageInfo: PageInfo!
edges: [UserEdge!]!
aggregate: Aggregate!
}
"#;
        let user_connection_types = connection_types_def(
            &sdml_parser::ast::Token::Ident("User", Span::new(0, 0)),
        )
        .unwrap();
        let actual_graphql_str = user_connection_types
            .into_iter()
            .fold("".to_string(), |acc, ty| {
                format!("{}{}", acc, ty.to_string())
            });
        assert_eq!(expected_graphql_str, actual_graphql_str)
    }
}
