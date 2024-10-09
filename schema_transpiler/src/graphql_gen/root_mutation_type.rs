//! Generates the root level mutation type

use super::*;

/// Code-gen root Mutation type with required OpenCRUD fields
/// to Create, Update / Upsert, Delete for all models.
/// ### Arguments.
///  * `models` - array of models in sdml.
/// ### Returns.
/// Root level mutation type definition.
pub(in crate::graphql_gen) fn root_mutation_type_def<'src>(
    models: Vec<&sdml_ast::ModelDecl<'src>>,
) -> GraphQLGenResult<Vec<TypeSystemDefinition>> {
    unimplemented!()
}

fn root_mutation_fields<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<Vec<FieldDefinition>> {
    let model_name = model_name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;

    unimplemented!()
}
