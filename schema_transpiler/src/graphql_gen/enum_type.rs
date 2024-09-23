use super::*;

/// Generates the GraphQL enum type for the given SDML enum.
pub fn enum_def<'src>(
    r#enum: &sdml_ast::EnumDecl<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    let enum_name = r#enum
        .name
        .try_get_graphql_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    Ok(TypeDefinition {
        extend: false,
        description: None,
        name: enum_name,
        directives: vec![],
        kind: TypeKind::Enum(EnumType {
            values: r#enum
                .elements
                .iter()
                .map(enum_value_def)
                .collect::<Result<Vec<_>, ErrorGraphQLGen>>()?,
        }),
    })
}

#[inline(always)]
fn enum_value_def<'src>(
    enum_val_tok: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<EnumValueDefinition> {
    let enum_value = enum_val_tok
        .try_get_graphql_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    Ok(EnumValueDefinition {
        description: None,
        value: enum_value,
        directives: vec![],
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chumsky::prelude::*;
    use sdml_parser::parser;

    use crate::graphql_gen::{enum_type::enum_def, ErrorGraphQLGen, TypeDefinition};

    #[test]
    fn test_enum_def() {
        let sdml_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/test_enum_def.sdml"
        ))
        .unwrap();
        let sdml_declarations = parser::delcarations()
            .parse(&sdml_str)
            .into_output()
            .expect("It should be a valid SDML.");
        let data_model = parser::semantic_analysis(sdml_declarations)
            .expect("A valid SDML file shouldn't fail in parsing.");
        let graphql_enums = data_model
            .enums()
            .values()
            .into_iter()
            .map(enum_def)
            .collect::<Result<Vec<TypeDefinition>, ErrorGraphQLGen>>()
            .unwrap();
        assert_eq!(data_model.enums().len(), graphql_enums.len());
        graphql_enums.iter().for_each(|graphql_enum| {
            assert!(
                data_model.enums().get(graphql_enum.name.as_str()).is_some(),
                "Unable to find {} enum",
                graphql_enum.name.as_str()
            );
        });
    }
}
