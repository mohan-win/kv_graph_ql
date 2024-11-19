//! Impelments code-gen functions for generating WhereUniqueInput filter type.
use super::*;

/// Generates WhereUniqueInput filter type for the given model.
/// When this fitler gets passed as an argument,
/// it will exactly match *at-most* 1 record in the graphQL response.
pub fn where_unique_unique_input_def<'src>(
    model: &sdml_ast::ModelDecl<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    // Note: SDML validates / ensures that only scalar fields can have @unique attribute
    // So we don't need to filter for scalar fields.
    let unique_scalar_fields = model
        .fields
        .iter()
        .filter(|fld| (fld.has_id_attrib() | fld.has_unique_attrib()));
    let unique_field_filters = unique_scalar_fields
        .map(unique_scalar_field_to_filter)
        .collect::<Result<Vec<InputValueDefinition>, ErrorGraphQLGen>>(
    )?;
    let model_name = model
        .name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?; // Note: Model should have a valid identifier, otherwise it will be caught by SDML parser! So we can just use unwrap()..
    Ok(TypeDefinition {
        extend: false,
        description: Some(
            "The where unique filter which can match at-most 1 object.".to_string(),
        ),
        name: open_crud_name::types::FilterInput::WhereUnique.name(model_name),
        directives: vec![],
        kind: TypeKind::InputObject(InputObjectType {
            fields: unique_field_filters,
        }),
    })
}

fn unique_scalar_field_to_filter<'src>(
    field: &sdml_ast::FieldDecl<'src>,
) -> GraphQLGenResult<InputValueDefinition> {
    debug_assert!(
        field.has_id_attrib() | field.has_unique_attrib(),
        "Only scalar fields with @id or @unique attribute should passed to this function"
    );
    let is_id_field = field.has_id_attrib();
    let ty = match &*field.field_type.r#type() {
        sdml_ast::Type::Enum { enum_ty_name } => {
            let ty_name = enum_ty_name
                .try_get_ident_name()
                .map_err(ErrorGraphQLGen::new_sdml_error)?;
            Ok(Type::new(ty_name, TypeMod::Optional))
        }
        sdml_ast::Type::Primitive {
            r#type: primitive_type,
            ..
        } => match primitive_type {
            sdml_ast::PrimitiveType::ShortStr if is_id_field => {
                Ok(open_crud_name::types::OpenCRUDType::IdType
                    .common_ty(TypeMod::Optional))
            }
            sdml_prim_type => {
                let graphql_ty_name =
                    Type::map_sdml_type_to_graphql_ty_name(sdml_prim_type);
                Ok(Type::new(&graphql_ty_name, TypeMod::Optional))
            }
        },
        other_type => {
            debug_assert!(
                false,
                "This shouldn't happen. Only scalar types should be allowed to be @unique in SDML"
            );
            let field_name = field
                .name
                .try_get_ident_name()
                .map_err(ErrorGraphQLGen::new_sdml_error)?;
            Err(ErrorGraphQLGen::SDMLError {
                error: format!(
                    "The type {other_type:?} of the field {field_name} is invalid for WhereUniqueInput"
                ),
                pos: field.name.span(),
            })
        }
    }?;
    Ok(InputValueDefinition {
        description: None,
        name: if is_id_field {
            open_crud_name::fields::Field::Id.common_name()
        } else {
            let field_name = field
                .name
                .try_get_graphql_name()
                .map_err(ErrorGraphQLGen::new_sdml_error)?;
            Name::new(field_name)
        },
        ty,
        default_value: None,
        directives: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use chumsky::prelude::*;
    use sdml_parser::parser;

    #[test]
    fn test_where_unique_def() {
        let mut expected_graphql_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/input_type/user_where_unique_input.graphql"
        ))
        .unwrap();
        expected_graphql_str.retain(|c| !c.is_whitespace());
        let sdml_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/input_type/user_where_unique_input.sdml"
        ))
        .unwrap();
        let sdml_declarations = parser::delcarations()
            .parse(&sdml_str)
            .into_output()
            .expect("It should be a valid SDML.");
        let data_model = parser::semantic_analysis(sdml_declarations)
            .expect("A valid SDML file shouldn't fail in parsing.");
        let user_model_sdml_ast = data_model
            .models()
            .get("User")
            .expect("User model should exist in the SDML.");
        let user_where_input_grapql_ast =
            where_unique_unique_input_def(user_model_sdml_ast)
                .expect("It should return UserWhereInput");
        let mut user_where_input_graphql = user_where_input_grapql_ast.to_string();
        user_where_input_graphql.retain(|c| !c.is_whitespace());
        assert_eq!(expected_graphql_str, user_where_input_graphql)
    }
}
