use super::*;

/// Code-gen for *update input types* for the given model.
/// It generates,
/// * {ModelName}UpdateInput,
/// * {ModelName}UpsertInput,
/// * {ModelName}UpdateOneInlineInput,
/// * {ModelName}UpdateManyInlineInput,
/// * {ModelName}UpdateWithNestedWhereUniqueInput,
/// * {ModelName}UpsertWithNestedWhereUniqueInput,
/// * {ModelName}ConnectInput.
pub(in crate::graphql_gen) fn update_input_types_def<'src>(
    model: &sdml_ast::ModelDecl<'src>,
) -> GraphQLGenResult<Vec<TypeDefinition>> {
    unimplemented!()
}

/// Code-gen for the input type use to update a object.
/// Ex. UserUpdateInput is used to capture
/// the *complete data* to update a single user object including contained relations.
fn update_input_def<'src>(
    model: &sdml_ast::ModelDecl<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    let model_fields = helpers::get_model_fields(
        model,
        // Note: Filter out relation_scalar fields & auto generated ids.
        // Because they are not updatable directly.
        true, true,
    );

    let mut input_field_defs = model_fields
        .non_relation_fields
        .into_iter()
        .map(non_relation_field_input_def)
        .collect::<GraphQLGenResult<Vec<InputValueDefinition>>>()?;
    let relation_input_field_defs = model_fields
        .relation_fields
        .into_iter()
        .map(relation_field_input_def)
        .collect::<GraphQLGenResult<Vec<InputValueDefinition>>>()?;
    input_field_defs.extend(relation_input_field_defs);

    let model_name = model
        .name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    Ok(TypeDefinition {
        extend: false,
        description: None,
        name: open_crud_name::UpdateInputType::UpdateInput.name(model_name),
        directives: vec![],
        kind: TypeKind::InputObject(InputObjectType {
            fields: input_field_defs,
        }),
    })
}

fn upsert_input_def<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    unimplemented!()
}

fn update_many_input_def<'src>(
    model: &sdml_ast::ModelDecl<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    unimplemented!()
}

fn update_one_inline_input_def<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    unimplemented!()
}

fn update_many_inline_input_def<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    unimplemented!()
}

fn update_with_nested_where_unique_input_def<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    unimplemented!()
}

fn upsert_with_nested_where_unique_input_def<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    unimplemented!()
}

fn connect_input_def<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    unimplemented!()
}

pub(in crate::graphql_gen) fn connection_position_input_def(
) -> GraphQLGenResult<TypeDefinition> {
    unimplemented!()
}

/// Code-gen input arg for the non-relation field.
fn non_relation_field_input_def<'src>(
    field: &sdml_ast::FieldDecl<'src>,
) -> GraphQLGenResult<InputValueDefinition> {
    let ty_str = match &*field.field_type.r#type() {
        sdml_ast::Type::Unknown(..) | sdml_ast::Type::Relation(..) => {
            Err(ErrorGraphQLGen::SDMLError {
                error: "Only non-relation field is allowed here.".to_string(),
                pos: field.name.span(),
            })
        }
        sdml_ast::Type::Primitive { r#type, .. } => {
            Ok(Type::map_sdml_type_to_graphql_ty_name(r#type))
        }
        sdml_ast::Type::Enum { enum_ty_name } => Ok(enum_ty_name
            .try_get_ident_name()
            .map_err(ErrorGraphQLGen::new_sdml_error)?
            .to_string()),
    }?;
    // Note: All the input args for update are optional!!
    let type_mod: TypeMod = match field.field_type.type_mod {
        sdml_ast::FieldTypeMod::Array => TypeMod::ArrayOptional,
        sdml_ast::FieldTypeMod::NonOptional | sdml_ast::FieldTypeMod::Optional => {
            TypeMod::Optional
        }
    };

    Ok(InputValueDefinition {
        description: None,
        name: field
            .name
            .try_get_graphql_name()
            .map_err(ErrorGraphQLGen::new_sdml_error)?,
        ty: Type::new(&ty_str, type_mod),
        default_value: None, // Note: The default value is set to None. The appropriate default
        directives: vec![],
    })
}

fn relation_field_input_def<'src>(
    field: &sdml_ast::FieldDecl<'src>,
) -> GraphQLGenResult<InputValueDefinition> {
    if let sdml_ast::Type::Relation(edge) = &*field.field_type.r#type() {
        let field_name = field
            .name
            .try_get_graphql_name()
            .map_err(ErrorGraphQLGen::new_sdml_error)?;
        let referenced_model_name = edge
            .referenced_model_name()
            .try_get_ident_name()
            .map_err(ErrorGraphQLGen::new_sdml_error)?;
        let field_ty = if field.field_type.is_array() {
            open_crud_name::UpdateInputType::UpdateManyInlineInput
                .ty(referenced_model_name, TypeMod::Optional)
        } else {
            open_crud_name::UpdateInputType::UpdateOneInlineInput
                .ty(referenced_model_name, TypeMod::Optional)
        };
        Ok(InputValueDefinition {
            description: None,
            name: field_name,
            ty: field_ty,
            default_value: None,
            directives: vec![],
        })
    } else {
        Err(ErrorGraphQLGen::SDMLError {
            error: "Only relation field is expected!".to_string(),
            pos: field.name.span(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::update_input_def;

    use chumsky::prelude::*;
    use sdml_parser::parser;
    use std::fs;

    #[test]
    fn test_user_update_input_def() {
        let mut expected_graphql_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/input_type/test_user_update_input_def.graphql"
        ))
        .unwrap();
        expected_graphql_str.retain(|c| !c.is_whitespace());

        let sdml_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/input_type/test_user_update_input_def.sdml"
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
        let update_user_input_graphql_ast = update_input_def(user_model_sdml_ast)
            .expect("It should return all 'update user input'.");

        let mut update_user_input_graphql_str = update_user_input_graphql_ast.to_string();
        eprintln!("{}", update_user_input_graphql_str);
        update_user_input_graphql_str.retain(|c| !c.is_whitespace());
        assert_eq!(expected_graphql_str, update_user_input_graphql_str)
    }
}
