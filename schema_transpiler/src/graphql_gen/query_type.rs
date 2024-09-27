//! Generates the root level query type.
use sdml_ast::ModelDecl;

use super::*;
use pluralizer;

/// Code-gen root Query type with required OpenCRUD fields
/// to query information for all the models.
/// ## Arguments
/// * models - array of models in sdml.
/// ## Returns
/// Root level query type definition.
pub fn query_type_def<'src>(
    models: &Vec<ModelDecl<'src>>,
) -> GraphQLGenResult<TypeDefinition> {
    unimplemented!()
}

fn root_node_field() -> GraphQLGenResult<FieldDefinition> {
    Ok(FieldDefinition {
        description: None,
        name: Name::new(
            open_crud::OpenCRUDType::Query(QueryType::RootNode).common_name(),
        ),
        arguments: vec![InputValueDefinition {
            description: None,
            name: Name::new(open_crud::Field::Id.common_name()),
            ty: open_crud::OpenCRUDType::Id
                .common_ty(sdml_ast::FieldTypeMod::NonOptional),
            default_value: None,
            directives: vec![],
        }],
        ty: open_crud::QueryType::RootNode.common_ty(sdml_ast::FieldTypeMod::Optional),
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
            name: Name::new(model_name),
            arguments: vec![InputValueDefinition {
                description: None,
                name: Name::new(model_name),
                ty: open_crud::FilterInputType::WhereUniqueInput
                    .ty(model_name, sdml_ast::FieldTypeMod::NonOptional),
                default_value: None,
                directives: vec![],
            }],
            ty: Type::new(model_name, sdml_ast::FieldTypeMod::Optional),
            directives: vec![],
        },
    );

    root_query_fields.push(
        // Query array of objects.
        FieldDefinition {
            description: None,
            name: Name::new(
                &pluralizer::pluralize(model_name, 2, false)
                    .to_ascii_lowercase()
                    .to_string(),
            ),
            arguments: r#type::array_field_args(model_name)?,
            ty: Type::new(model_name, sdml_ast::FieldTypeMod::Array),
            directives: vec![],
        },
    );

    root_query_fields.push(
        // Query object connection for multiple objects.
        FieldDefinition {
            description: None,
            name: Name::new(open_crud::QueryField::Connection.named(model_name)),
            arguments: r#type::array_field_args(model_name)?,
            ty: open_crud::QueryType::Auxiliary(AuxiliaryType::Connection)
                .ty(model_name, sdml_ast::FieldTypeMod::NonOptional),
            directives: vec![],
        },
    );

    Ok(root_query_fields)
}
