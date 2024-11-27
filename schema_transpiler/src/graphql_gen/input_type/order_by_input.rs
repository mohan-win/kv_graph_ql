use super::*;

/// Generates OrderByInput enum for the given model's scalar fields.
pub fn order_by_input_enum_def<'src>(
  model: &sdml_ast::ModelDecl<'src>,
) -> GraphQLGenResult<TypeDefinition> {
  let mut scalar_fields = model.fields.iter().filter(|fld| fld.field_type.is_scalar());
  let order_by_elements = scalar_fields.try_fold(Vec::new(), |mut acc, fld| {
    let field_name = if !fld.has_id_attrib() {
      fld
        .name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?
    } else {
      &open_crud_name::fields::Field::Id.common_name()
    };

    acc.push(EnumValueDefinition {
      description: None,
      value: Name::new(format!("{field_name}_ASC")),
      directives: vec![],
    });
    acc.push(EnumValueDefinition {
      description: None,
      value: Name::new(format!("{field_name}_DSC")),
      directives: vec![],
    });
    Ok(acc)
  })?;
  let model_name = model
    .name
    .try_get_ident_name()
    .map_err(ErrorGraphQLGen::new_sdml_error)?;
  Ok(TypeDefinition {
    extend: false,
    description: Some(format!("Order by input for {model_name}'s scalar fields")),
    name: open_crud_name::types::OpenCRUDType::OrderByInput.name(model_name),
    directives: vec![],
    kind: TypeKind::Enum(EnumType {
      values: order_by_elements,
    }),
  })
}

#[cfg(test)]
mod tests {
  use chumsky::prelude::*;
  use sdml_parser::parser;
  use std::fs;

  use super::*;

  #[test]
  fn test_order_by_input_enum_def() {
    let mut expected_graphql_str = fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/input_type/user_order_by_input.graphql"
    ))
    .unwrap();
    expected_graphql_str.retain(|c| !c.is_whitespace());
    let sdml_str = fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/input_type/user_order_by_input.sdml"
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
    let user_order_by_input_grapql_ast = order_by_input_enum_def(user_model_sdml_ast)
      .expect("It should return UserOrderByInput");
    let mut user_order_by_input_graphql = user_order_by_input_grapql_ast.to_string();
    user_order_by_input_graphql.retain(|c| !c.is_whitespace());
    assert_eq!(expected_graphql_str, user_order_by_input_graphql)
  }
}
