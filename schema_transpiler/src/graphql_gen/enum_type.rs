use super::*;

/// Generates the GraphQL enum type for the given SDML enum.
pub fn enum_def<'src>(
    model: &sdml_ast::EnumDecl<'src>,
) -> GraphQLGenResult<TypeDefinition> {
}
