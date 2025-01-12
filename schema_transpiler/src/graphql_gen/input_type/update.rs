use sdml_ast::ModelDecl;

use super::*;

/// Code-gen for *update input types* for the given model.
/// It generates,
/// * {ModelName}UpdateInput,
/// * {ModelName}UpsertInput,
/// * {ModelName}UpdateOneInlineInput,
/// * {ModelName}UpdateManyInlineInput [if relevant].,
/// * {ModelName}UpdateWithNestedWhereUniqueInput,
/// * {ModelName}UpsertWithNestedWhereUniqueInput,
/// * {ModelName}ConnectInput.
pub fn update_input_types_def(
  model: &sdml_ast::ModelDecl,
) -> GraphQLGenResult<Vec<TypeDefinition>> {
  let mut result = Vec::new();
  result.push(update_input_def(model)?);
  result.push(upsert_input_def(&model.name)?);
  update_many_input_def(model)?.map(|input_type| result.push(input_type));
  result.push(update_one_inline_input_def(&model.name)?);
  result.push(update_many_inline_input_def(&model.name)?);
  result.push(update_with_nested_where_unique_input_def(&model.name)?);
  result.push(upsert_with_nested_where_unique_input_def(&model.name)?);
  result.push(connect_input_def(&model.name)?);
  Ok(result)
}

/// Code-gen for the input type use to update a object.
/// Ex. UserUpdateInput is used to capture
/// the *complete data* to update a single user object including contained relations.
fn update_input_def(model: &sdml_ast::ModelDecl) -> GraphQLGenResult<TypeDefinition> {
  let model_fields = model.get_fields();
  // Note: Filter out relation_scalar fields & ids.
  // Because they are not updatable directly.
  // But UpdateInput can be used to update unique fields.
  // [see] update_many_input_def() where unique fields are filtered out
  let mut non_relation_fields = Vec::new();
  non_relation_fields.extend(&model_fields.unique);
  non_relation_fields
    .extend(model_fields.get_rest(sdml_ast::ModelIndexedFieldsFilter::All));

  let mut input_field_defs = non_relation_fields
    .into_iter()
    .map(non_relation_field_input_def)
    .collect::<GraphQLGenResult<Vec<InputValueDefinition>>>()?;
  let relation_input_field_defs = model_fields
    .relation
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

fn upsert_input_def(model_name: &sdml_ast::Token) -> GraphQLGenResult<TypeDefinition> {
  let model_name = model_name
    .try_get_ident_name()
    .map_err(ErrorGraphQLGen::new_sdml_error)?;
  let fields = vec![
    // create
    InputValueDefinition {
      description: None,
      name: open_crud_name::fields::UpdateInputArg::Create.common_name(),
      ty: open_crud_name::types::CreateInput::Create.ty(model_name, TypeMod::NonOptional), // Note: Non-Optional
      default_value: None,
      directives: vec![],
    },
    // update
    InputValueDefinition {
      description: None,
      name: open_crud_name::fields::UpdateInputArg::Update.common_name(),
      ty: open_crud_name::types::UpdateInput::Update.ty(model_name, TypeMod::NonOptional), // Note:Non-Optional
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

/// Returns `true` if updateMany API and type is relevant for the model.
/// Ex. If Model has only @unique & @id fields, then updateMany API won't be
/// relevant for it.
pub fn has_update_many_input(model: &ModelDecl) -> bool {
  let fields = model.get_fields();
  fields
    .get_rest(sdml_ast::ModelIndexedFieldsFilter::All)
    .len()
    > 0
}

/// Code-gen the input type used to update many objects in one go..
/// Ex. UserUpdateManyInput is used to capture data to update many User objects in one go.
/// **Note:**
/// * We need to filter out, id, unique fields and relation fields, because
/// they are not updatable with update_many interface.
/// * If there are no fields, this function will return None.
///
fn update_many_input_def(
  model: &sdml_ast::ModelDecl,
) -> GraphQLGenResult<Option<TypeDefinition>> {
  let model_fields = model.get_fields();

  // Note: Filter out relation_scalar fields & ids.
  // and [important] also filter out unique fields.
  // Because they are not updatable directly in UpdateManyInput.
  let non_unique_field_defs = model_fields
    .get_rest(sdml_ast::ModelIndexedFieldsFilter::All)
    .into_iter()
    .map(non_relation_field_input_def)
    .collect::<GraphQLGenResult<Vec<InputValueDefinition>>>()?;

  if non_unique_field_defs.len() == 0 {
    Ok(None)
  } else {
    let model_name = model
      .name
      .try_get_ident_name()
      .map_err(ErrorGraphQLGen::new_sdml_error)?;
    Ok(Some(TypeDefinition {
      extend: false,
      description: None,
      name: open_crud_name::types::UpdateInput::UpdateMany.name(model_name),
      directives: vec![],
      kind: TypeKind::InputObject(InputObjectType {
        fields: non_unique_field_defs,
      }),
    }))
  }
}

fn update_one_inline_input_def(
  model_name: &sdml_ast::Token,
) -> GraphQLGenResult<TypeDefinition> {
  let model_name = model_name
    .try_get_ident_name()
    .map_err(ErrorGraphQLGen::new_sdml_error)?;
  let fields = vec![
    // create
    InputValueDefinition {
      description: Some(format!("Create and connect a new '{}' object.", model_name)),
      name: open_crud_name::fields::UpdateInputArg::Create.common_name(),
      ty: open_crud_name::types::CreateInput::Create.ty(model_name, TypeMod::Optional),
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

fn update_many_inline_input_def(
  model_name: &sdml_ast::Token,
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

fn update_with_nested_where_unique_input_def(
  model_name: &sdml_ast::Token,
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
      ty: open_crud_name::types::UpdateInput::Update.ty(model_name, TypeMod::NonOptional), // Note:Non-Optional
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

fn upsert_with_nested_where_unique_input_def(
  model_name: &sdml_ast::Token,
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
      ty: open_crud_name::types::UpdateInput::Upsert.ty(model_name, TypeMod::NonOptional), // Note:Non-Optional
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

fn connect_input_def(model_name: &sdml_ast::Token) -> GraphQLGenResult<TypeDefinition> {
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

pub fn connect_position_input_def() -> GraphQLGenResult<TypeDefinition> {
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
fn non_relation_field_input_def(
  field: &sdml_ast::FieldDecl,
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
    sdml_ast::Type::Enum { enum_ty_name } => Ok(
      enum_ty_name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?
        .to_string(),
    ),
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

fn relation_field_input_def(
  field: &sdml_ast::FieldDecl,
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
  use crate::graphql_gen::TypeDefinition;

  use super::connect_position_input_def;
  use super::update_input_types_def;
  use sdml_parser;
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
    let data_model = sdml_parser::parse(&sdml_str)
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
  fn test_update_input_types_def() {
    let mut expected_graphql_str = fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/input_type/test_update_input_types_def.graphql"
    ))
    .unwrap();
    expected_graphql_str.retain(|c| !c.is_whitespace());

    let sdml_str = fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/input_type/test_update_input_types_def.sdml"
    ))
    .unwrap();
    let data_model = sdml_parser::parse(&sdml_str)
      .expect("A valid SDML file shouldn't fail in parsing.");

    let update_input_types_graphql_ast = data_model
      .models_sorted()
      .iter()
      .flat_map(|model| {
        update_input_types_def(model).expect("update_input_types_def should succeed!")
      })
      .collect::<Vec<TypeDefinition>>();

    let mut update_user_input_graphql_str = update_input_types_graphql_ast
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
