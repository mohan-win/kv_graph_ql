//! Generates the root level query type.
use sdml_ast::ModelDecl;

use super::*;

/// Code-gen root Query type with required OpenCRUD fields
/// to query information for all the models.
/// ## Arguments
/// * models - array of models in sdml.
/// ## Returns
/// Root level query type definition.
pub fn query_type_def<'src>(
    models: &Vec<ModelDecl<'src>>,
) -> GraphQLGenResult<TypeDefinition> {
}

/// Return root level query fields for given model.
pub fn root_query_fields<'src>(
    model_name: &Token<'src>,
) -> GraphQLGenResult<FieldDefinition> {
    
}
