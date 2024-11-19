//! Generates the root level query type.
use sdml_ast::ModelDecl;

use super::*;

/// Code-gen root Query type with required OpenCRUD fields
/// to query information for all the models.
/// ### Arguments
/// * models - array of models in sdml.
/// ## Returns
/// Root level query type definition.
pub fn root_query_type_def<'src>(
    models: &Vec<&ModelDecl<'src>>,
) -> GraphQLGenResult<TypeDefinition> {
    let mut fields = Vec::new();
    fields.push(root_node_field()?);
    let fields = models.iter().try_fold(fields, |mut acc, model| {
        acc.extend(root_query_fields(&model.name)?);
        Ok(acc)
    })?;
    Ok(TypeDefinition {
        extend: false,
        description: None,
        name: open_crud_name::types::QueryType::RootQuery.common_name(),
        directives: vec![],
        kind: TypeKind::Object(ObjectType {
            implements: vec![],
            fields,
        }),
    })
}

fn root_node_field() -> GraphQLGenResult<FieldDefinition> {
    Ok(FieldDefinition {
        description: None,
        name: open_crud_name::fields::QueryType::RootNodeField.common_name(),
        arguments: vec![InputValueDefinition {
            description: None,
            name: open_crud_name::fields::Field::Id.common_name(),
            ty: open_crud_name::types::OpenCRUDType::IdType
                .common_ty(TypeMod::NonOptional),
            default_value: None,
            directives: vec![],
        }],
        ty: open_crud_name::types::QueryType::RootNode.common_ty(TypeMod::Optional),
        directives: vec![],
    })
}

/// Return root level query fields for given model.
fn root_query_fields<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<Vec<FieldDefinition>> {
    let model_name = model_name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    let mut root_query_fields = vec![];

    root_query_fields.push(
        // Query unique object.
        FieldDefinition {
            description: None,
            name: open_crud_name::fields::QueryType::RootField.name(model_name),
            arguments: vec![InputValueDefinition {
                description: None,
                name: open_crud_name::fields::QueryInputArg::Where.common_name(),
                ty: open_crud_name::types::FilterInput::WhereUnique
                    .ty(model_name, TypeMod::NonOptional),
                default_value: None,
                directives: vec![],
            }],
            ty: Type::new(model_name, TypeMod::Optional),
            directives: vec![],
        },
    );

    root_query_fields.push(
        // Query array of objects.
        FieldDefinition {
            description: None,
            name: open_crud_name::fields::QueryType::RootFieldArray.name(model_name),
            arguments: r#type::array_field_args(model_name)?,
            ty: Type::new(model_name, TypeMod::Array),
            directives: vec![],
        },
    );

    root_query_fields.push(
        // Query object connection for multiple objects.
        FieldDefinition {
            description: None,
            name: open_crud_name::fields::QueryType::RootFieldConnection.name(model_name),
            arguments: r#type::array_field_args(model_name)?,
            ty: open_crud_name::types::AuxiliaryType::Connection
                .ty(model_name, TypeMod::NonOptional),
            directives: vec![],
        },
    );

    Ok(root_query_fields)
}

#[cfg(test)]
mod test {
    use chumsky::prelude::*;
    use sdml_parser::parser;

    use std::fs;
    #[test]
    fn test_root_query_type_def() {
        let sdml_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/test_root_query_type_def.sdml"
        ))
        .unwrap();
        let mut expected_graphql_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/test_root_query_type_def.graphql"
        ))
        .unwrap();
        expected_graphql_str.retain(|c| !c.is_whitespace());

        let sdml_ast = parser::delcarations()
            .parse(&sdml_str)
            .into_result()
            .expect("It should be a valid sdml file");
        let sdml_ast = parser::semantic_analysis(sdml_ast)
            .expect("Semantic analysis should succeed!");
        let root_query_type =
            super::root_query_type_def(&sdml_ast.models_sorted()).unwrap();
        let mut actual_graphql_str = root_query_type.to_string();
        actual_graphql_str.retain(|c| !c.is_whitespace());
        assert_eq!(expected_graphql_str, actual_graphql_str);
    }
}
