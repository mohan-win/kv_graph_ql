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
    Ok(vec![
        update_input_def(model)?,
        upsert_input_def(&model.name)?,
        update_many_input_def(model)?,
        update_one_inline_input_def(&model.name)?,
        update_many_inline_input_def(&model.name)?,
        update_with_nested_where_unique_input_def(&model.name)?,
        upsert_with_nested_where_unique_input_def(&model.name)?,
        connect_input_def(&model.name)?,
    ])
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
        // But UpdateInput can be used to update unique fields.
        // [see] update_many_input_def() where unique fields are filtered out
        true, true, false,
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
        name: open_crud_name::types::UpdateInput::Update.name(model_name),
        directives: vec![],
        kind: TypeKind::InputObject(InputObjectType {
            fields: input_field_defs,
        }),
    })
}

fn upsert_input_def<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    let model_name = model_name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    let fields = vec![
        // create
        InputValueDefinition {
            description: None,
            name: open_crud_name::fields::UpdateInputArg::Create.common_name(),
            ty: open_crud_name::types::CreateInput::Create
                .ty(model_name, TypeMod::NonOptional), // Note: Non-Optional
            default_value: None,
            directives: vec![],
        },
        // update
        InputValueDefinition {
            description: None,
            name: open_crud_name::fields::UpdateInputArg::Update.common_name(),
            ty: open_crud_name::types::UpdateInput::Update
                .ty(model_name, TypeMod::NonOptional), // Note:Non-Optional
            default_value: None,
            directives: vec![],
        },
    ];
    Ok(TypeDefinition {
        extend: false,
        description: None,
        name: open_crud_name::types::UpdateInput::Upsert.name(model_name),
        directives: vec![],
        kind: TypeKind::InputObject(InputObjectType { fields }),
    })
}

/// Code-gen the input type used to update many objects in one go..
/// Ex. UserUpdateManyInput is used to capture data to update many objects in one go.
/// **Note:** We need to filter out, id, unique fields and relation fields, because
/// they are not updatable with update_many interface.
fn update_many_input_def<'src>(
    model: &sdml_ast::ModelDecl<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    let model_fields = helpers::get_model_fields(
        model,
        // Note: Filter out relation_scalar fields & auto generated ids.
        // and [important] also filter out unique fields.
        // Because they are not updatable directly in UpdateManyInput.
        true, true, true,
    );

    let non_relation_input_field_defs = model_fields
        .non_relation_fields
        .into_iter()
        .map(non_relation_field_input_def)
        .collect::<GraphQLGenResult<Vec<InputValueDefinition>>>()?;
    // Note: relation_input_fields can't be present in UpdateManyInput.

    let model_name = model
        .name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    Ok(TypeDefinition {
        extend: false,
        description: None,
        name: open_crud_name::types::UpdateInput::UpdateMany.name(model_name),
        directives: vec![],
        kind: TypeKind::InputObject(InputObjectType {
            fields: non_relation_input_field_defs,
        }),
    })
}

fn update_one_inline_input_def<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    let model_name = model_name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    let fields = vec![
        // create
        InputValueDefinition {
            description: Some(format!(
                "Create and connect a new '{}' object.",
                model_name
            )),
            name: open_crud_name::fields::UpdateInputArg::Create.common_name(),
            ty: open_crud_name::types::CreateInput::Create
                .ty(model_name, TypeMod::Optional),
            default_value: None,
            directives: vec![],
        },
        // update
        InputValueDefinition {
            description: Some(format!("Update '{}' object if exists.", model_name)),
            name: open_crud_name::fields::UpdateInputArg::Update.common_name(),
            ty: open_crud_name::types::UpdateInput::UpdateWithNestedWhereUnique
                .ty(model_name, TypeMod::Optional),
            default_value: None,
            directives: vec![],
        },
        // upsert
        InputValueDefinition {
            description: Some(format!("Upsert '{}' object.", model_name)),
            name: open_crud_name::fields::UpdateInputArg::Upsert.common_name(),
            ty: open_crud_name::types::UpdateInput::UpsertWithNestedWhereUnique
                .ty(model_name, TypeMod::Optional),
            default_value: None,
            directives: vec![],
        },
        // connect
        InputValueDefinition {
            description: Some(format!("Connect an existing '{}' object.", model_name)),
            name: open_crud_name::fields::UpdateInputArg::Connect.common_name(),
            ty: open_crud_name::types::FilterInput::WhereUnique
                .ty(model_name, TypeMod::Optional),
            default_value: None,
            directives: vec![],
        },
        // disconnect
        InputValueDefinition {
            description: Some(format!("Disconnect '{}' object.", model_name)),
            name: open_crud_name::fields::UpdateInputArg::Disconnect.common_name(),
            ty: Type::new_from_str(FIELD_TYPE_NAME_BOOL).unwrap(),
            default_value: None,
            directives: vec![],
        },
        // delete
        InputValueDefinition {
            description: Some(format!("Delete '{}' object.", model_name)),
            name: open_crud_name::fields::UpdateInputArg::Delete.common_name(),
            ty: Type::new_from_str(FIELD_TYPE_NAME_BOOL).unwrap(),
            default_value: None,
            directives: vec![],
        },
    ];
    Ok(TypeDefinition {
        extend: false,
        description: None,
        name: open_crud_name::types::UpdateInput::UpdateOneInline.name(model_name),
        directives: vec![],
        kind: TypeKind::InputObject(InputObjectType { fields }),
    })
}

fn update_many_inline_input_def<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    let model_name = model_name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    let fields = vec![
        // create
        InputValueDefinition {
            description: Some(format!(
                "Create and connect multiple new '{}' objects.",
                model_name
            )),
            name: open_crud_name::fields::UpdateInputArg::Create.common_name(),
            ty: open_crud_name::types::CreateInput::Create
                .ty(model_name, TypeMod::ArrayOptional),
            default_value: None,
            directives: vec![],
        },
        // update
        InputValueDefinition {
            description: Some(format!(
                "Update multiple '{}' objects if exists.",
                model_name
            )),
            name: open_crud_name::fields::UpdateInputArg::Update.common_name(),
            ty: open_crud_name::types::UpdateInput::UpdateWithNestedWhereUnique
                .ty(model_name, TypeMod::ArrayOptional),
            default_value: None,
            directives: vec![],
        },
        // upsert
        InputValueDefinition {
            description: Some(format!("Upsert multiple '{}' objects.", model_name)),
            name: open_crud_name::fields::UpdateInputArg::Upsert.common_name(),
            ty: open_crud_name::types::UpdateInput::UpsertWithNestedWhereUnique
                .ty(model_name, TypeMod::ArrayOptional),
            default_value: None,
            directives: vec![],
        },
        // connect
        InputValueDefinition {
            description: Some(format!(
                "Connect multiple existing '{}' objects.",
                model_name
            )),
            name: open_crud_name::fields::UpdateInputArg::Connect.common_name(),
            ty: open_crud_name::types::UpdateInput::Connect
                .ty(model_name, TypeMod::ArrayOptional),
            default_value: None,
            directives: vec![],
        },
        // set
        InputValueDefinition {
            description: Some(format!(
                "Replace existing relation with multiple '{}' objects.",
                model_name
            )),
            name: open_crud_name::fields::UpdateInputArg::Set.common_name(),
            ty: open_crud_name::types::FilterInput::WhereUnique
                .ty(model_name, TypeMod::ArrayOptional),
            default_value: None,
            directives: vec![],
        },
        // disconnect
        InputValueDefinition {
            description: Some(format!(
                "Disconnect multiple '{}' objects from relation.",
                model_name
            )),
            name: open_crud_name::fields::UpdateInputArg::Disconnect.common_name(),
            ty: open_crud_name::types::FilterInput::WhereUnique
                .ty(model_name, TypeMod::ArrayOptional),
            default_value: None,
            directives: vec![],
        },
        // delete
        InputValueDefinition {
            description: Some(format!("Delete multiple '{}' objects.", model_name)),
            name: open_crud_name::fields::UpdateInputArg::Delete.common_name(),
            ty: open_crud_name::types::FilterInput::WhereUnique
                .ty(model_name, TypeMod::ArrayOptional),
            default_value: None,
            directives: vec![],
        },
    ];
    Ok(TypeDefinition {
        extend: false,
        description: None,
        name: open_crud_name::types::UpdateInput::UpdateManyInline.name(model_name),
        directives: vec![],
        kind: TypeKind::InputObject(InputObjectType { fields }),
    })
}

fn update_with_nested_where_unique_input_def<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    let model_name = model_name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    let fields = vec![
        // where
        InputValueDefinition {
            description: None,
            name: open_crud_name::fields::UpdateInputArg::Where.common_name(),
            ty: open_crud_name::types::FilterInput::WhereUnique
                .ty(model_name, TypeMod::NonOptional), // Note: Non-Optional
            default_value: None,
            directives: vec![],
        },
        // data
        InputValueDefinition {
            description: None,
            name: open_crud_name::fields::UpdateInputArg::Data.common_name(),
            ty: open_crud_name::types::UpdateInput::Update
                .ty(model_name, TypeMod::NonOptional), // Note:Non-Optional
            default_value: None,
            directives: vec![],
        },
    ];
    Ok(TypeDefinition {
        extend: false,
        description: None,
        name: open_crud_name::types::UpdateInput::UpdateWithNestedWhereUnique
            .name(model_name),
        directives: vec![],
        kind: TypeKind::InputObject(InputObjectType { fields }),
    })
}

fn upsert_with_nested_where_unique_input_def<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    let model_name = model_name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    let fields = vec![
        // where
        InputValueDefinition {
            description: None,
            name: open_crud_name::fields::UpdateInputArg::Where.common_name(),
            ty: open_crud_name::types::FilterInput::WhereUnique
                .ty(model_name, TypeMod::NonOptional), // Note: Non-Optional
            default_value: None,
            directives: vec![],
        },
        // data
        InputValueDefinition {
            description: None,
            name: open_crud_name::fields::UpdateInputArg::Data.common_name(),
            ty: open_crud_name::types::UpdateInput::Upsert
                .ty(model_name, TypeMod::NonOptional), // Note:Non-Optional
            default_value: None,
            directives: vec![],
        },
    ];
    Ok(TypeDefinition {
        extend: false,
        description: None,
        name: open_crud_name::types::UpdateInput::UpsertWithNestedWhereUnique
            .name(model_name),
        directives: vec![],
        kind: TypeKind::InputObject(InputObjectType { fields }),
    })
}

fn connect_input_def<'src>(
    model_name: &sdml_ast::Token<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    let model_name = model_name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    let fields = vec![
        // where
        InputValueDefinition {
            description: Some(format!("'{}' object to connect", model_name)),
            name: open_crud_name::fields::UpdateInputArg::Where.common_name(),
            ty: open_crud_name::types::FilterInput::WhereUnique
                .ty(model_name, TypeMod::NonOptional), // Note: Non-Optional
            default_value: None,
            directives: vec![],
        },
        // position
        InputValueDefinition {
            description: Some("Specify the position in the list of connected objects, by-defult will add it to end of the list.".to_string()),
            name: open_crud_name::fields::UpdateInputArg::ConnectPosition.common_name(),
            ty: open_crud_name::types::UpdateInput::ConnectPosition.common_ty(TypeMod::Optional),
            default_value: None,
            directives: vec![],
        },
    ];
    Ok(TypeDefinition {
        extend: false,
        description: None,
        name: open_crud_name::types::UpdateInput::Connect.name(model_name),
        directives: vec![],
        kind: TypeKind::InputObject(InputObjectType { fields }),
    })
}

pub(in crate::graphql_gen) fn connect_position_input_def(
) -> GraphQLGenResult<TypeDefinition> {
    let fields = vec![
        // after
        InputValueDefinition {
            description: Some("Connect after the speficied ID.".to_string()),
            name: open_crud_name::fields::ConnectPositionInputArg::After.common_name(),
            ty: open_crud_name::types::OpenCRUDType::IdType.common_ty(TypeMod::Optional),
            default_value: None,
            directives: vec![],
        },
        // before
        InputValueDefinition {
            description: Some("Connect before the speficied ID.".to_string()),
            name: open_crud_name::fields::ConnectPositionInputArg::Before.common_name(),
            ty: open_crud_name::types::OpenCRUDType::IdType.common_ty(TypeMod::Optional),
            default_value: None,
            directives: vec![],
        },
        // start
        InputValueDefinition {
            description: Some("Connect at the first position.".to_string()),
            name: open_crud_name::fields::ConnectPositionInputArg::Start.common_name(),
            ty: Type::new(FIELD_TYPE_NAME_BOOL, TypeMod::Optional),
            default_value: None,
            directives: vec![],
        },
        // end
        InputValueDefinition {
            description: Some("Connect at the last position [default].".to_string()),
            name: open_crud_name::fields::ConnectPositionInputArg::End.common_name(),
            ty: Type::new(FIELD_TYPE_NAME_BOOL, TypeMod::Optional),
            default_value: None,
            directives: vec![],
        },
    ];
    Ok(TypeDefinition {
        extend: false,
        description: None,
        name: open_crud_name::types::UpdateInput::ConnectPosition.common_name(),
        directives: vec![],
        kind: TypeKind::InputObject(InputObjectType { fields }),
    })
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
            open_crud_name::types::UpdateInput::UpdateManyInline
                .ty(referenced_model_name, TypeMod::Optional)
        } else {
            open_crud_name::types::UpdateInput::UpdateOneInline
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
    use super::connect_position_input_def;
    use super::update_input_types_def;

    use chumsky::prelude::*;
    use sdml_parser::parser;
    use std::fs;

    #[test]
    fn test_user_update_input_types_def() {
        let mut expected_graphql_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/input_type/test_user_update_input_types_def.graphql"
        ))
        .unwrap();
        expected_graphql_str.retain(|c| !c.is_whitespace());

        let sdml_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/input_type/test_user_update_input_types_def.sdml"
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
        let update_user_input_graphql_ast = update_input_types_def(user_model_sdml_ast)
            .expect("It should return all 'update user input'.");

        let mut update_user_input_graphql_str = update_user_input_graphql_ast
            .into_iter()
            .fold("".to_string(), |acc, graphql_ast| {
                format!("{}{}", acc, graphql_ast)
            });
        update_user_input_graphql_str.retain(|c| !c.is_whitespace());
        assert_eq!(expected_graphql_str, update_user_input_graphql_str)
    }

    #[test]
    fn test_connect_position_input_def() {
        let mut expected_graphql_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/input_type/connect_position_input_def.graphql"
        ))
        .unwrap();
        expected_graphql_str.retain(|c| !c.is_whitespace());

        let mut connect_position_graphql_str =
            connect_position_input_def().unwrap().to_string();
        connect_position_graphql_str.retain(|c| !c.is_whitespace());

        assert_eq!(expected_graphql_str, connect_position_graphql_str);
    }
}
