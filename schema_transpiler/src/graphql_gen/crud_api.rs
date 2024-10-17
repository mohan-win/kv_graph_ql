//! Code-gen for OpenCRUD query API.

use super::*;

/// Generate type system definitions for CRUD apis as per OpenCRUD spec.
pub(in crate::graphql_gen) fn crud_api_def<'src>(
    data_model: &sdml_ast::DataModel<'src>,
) -> GraphQLGenResult<Vec<TypeSystemDefinition>> {
    let mut api_type_defs = Vec::new();
    // Custom Scalars.
    api_type_defs.push(TypeSystemDefinition::Type(scalar_date_time_def()));
    // Custom Directives.
    api_type_defs.push(TypeSystemDefinition::Directive(directive_map_def()));
    api_type_defs.push(TypeSystemDefinition::Directive(directive_unique_def()));
    api_type_defs.push(TypeSystemDefinition::Directive(directive_indexed_def()));

    // Root query type.
    api_type_defs.push(TypeSystemDefinition::Type(
        root_query_type::root_query_type_def(&data_model.models_sorted())?,
    ));
    // Root mutation type
    api_type_defs.push(TypeSystemDefinition::Type(
        root_mutation_type::root_mutation_type_def(&data_model.models_sorted())?,
    ));

    // Root Node interface.
    api_type_defs.push(TypeSystemDefinition::Type(interface_node_def()));
    // Enums
    let mut api_type_defs =
        data_model
            .enums_sorted()
            .iter()
            .try_fold(api_type_defs, |mut acc, r#enum| {
                acc.push(TypeSystemDefinition::Type(enum_type::enum_def(r#enum)?));
                Ok(acc)
            })?;
    // Common types.
    api_type_defs.push(TypeSystemDefinition::Type(aux_type::page_info_type_def()?));
    api_type_defs.push(TypeSystemDefinition::Type(aux_type::aggregage_type_def()?));
    api_type_defs.push(TypeSystemDefinition::Type(
        input_type::update::connect_position_input_def()?,
    ));

    // Model specific types & Models.
    data_model.models_sorted().iter().try_for_each(|model| {
        // Filters & Order_By
        api_type_defs.push(TypeSystemDefinition::Type(
            input_type::filter::r#where::where_input_def(model)?,
        ));
        api_type_defs.push(TypeSystemDefinition::Type(
            input_type::filter::where_unique::where_unique_unique_input_def(model)?,
        ));
        api_type_defs.push(TypeSystemDefinition::Type(
            input_type::order_by_input::order_by_input_enum_def(model)?,
        ));

        // Type & its aux type
        api_type_defs.extend(
            r#type::type_and_aux_types_def(model)?
                .into_iter()
                .map(TypeSystemDefinition::Type),
        );

        // Create input types
        api_type_defs.extend(
            input_type::create::create_input_types_def(model)?
                .into_iter()
                .map(TypeSystemDefinition::Type),
        );
        // Update input types
        api_type_defs.extend(
            input_type::update::update_input_types_def(model)?
                .into_iter()
                .map(TypeSystemDefinition::Type),
        );

        Ok(())
    })?;

    Ok(api_type_defs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chumsky::prelude::*;
    use sdml_parser::parser;
    use std::fs;

    #[test]
    fn test_crud_api_def() {
        let sdml_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/test_crud_api_def.sdml"
        ))
        .unwrap();
        let mut expected_graphql_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/test_crud_api_def.graphql"
        ))
        .unwrap();
        expected_graphql_str.retain(|c| !c.is_whitespace());

        let sdml_decls = parser::delcarations()
            .parse(&sdml_str)
            .into_result()
            .unwrap();
        let sdml_ast = parser::semantic_analysis(sdml_decls).unwrap();
        let query_api = crud_api_def(&sdml_ast).unwrap();
        let mut actual_query_api_graphql_str =
            query_api.iter().fold("".to_string(), |acc, graphql_ty| {
                format!("{}{}", acc, graphql_ty.to_string())
            });
        actual_query_api_graphql_str.retain(|c| !c.is_whitespace());
        assert_eq!(expected_graphql_str, actual_query_api_graphql_str);
    }
}
