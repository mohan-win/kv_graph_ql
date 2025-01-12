//! Implementes the code-gen functions for generating WhereInput filter type.
use super::*;

/// Generates where input type for the given model.
pub fn where_input_def(model: &sdml_ast::ModelDecl) -> GraphQLGenResult<TypeDefinition> {
  let mut filters = logical_operations_def(&model.name)?;
  let model_field_filters = model.fields.iter().map(field_to_filters).try_fold(
    Vec::new(),
    |mut acc, filters| match filters {
      Ok(filters) => {
        acc.extend(filters.into_iter());
        Ok(acc)
      }
      Err(e) => Err(e),
    },
  )?;
  filters.extend(model_field_filters.into_iter());
  let model_name = model
    .name
    .try_get_ident_name()
    .map_err(ErrorGraphQLGen::new_sdml_error)?;
  Ok(TypeDefinition {
    extend: false,
    description: Some(
      "The where filter which can match zero or more objects".to_string(),
    ),
    name: open_crud_name::types::FilterInput::Where.name(model_name),
    directives: vec![],
    kind: TypeKind::InputObject(InputObjectType { fields: filters }),
  })
}

/// Returns relevant filter arguments for the given field.
fn field_to_filters(
  field: &sdml_ast::FieldDecl,
) -> GraphQLGenResult<Vec<InputValueDefinition>> {
  let field_type = &*field.field_type.r#type();
  match field_type {
    sdml_ast::Type::Unknown(_) => {
      let field_name = field
        .name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
      Err(ErrorGraphQLGen::SDMLError {
        error: format!("The type of the field {field_name} is unknown"),
        pos: field.name.span(),
      })
    }
    sdml_ast::Type::Relation(_) => relation_field_def(&field.name, &field.field_type),
    sdml_ast::Type::Enum { .. } => enum_field_def(&field.name, field_type),
    sdml_ast::Type::Primitive {
      r#type: primitive_type,
      ..
    } => match *primitive_type {
      sdml_ast::PrimitiveType::ShortStr if field.has_id_attrib() => {
        id_field_def(&field.name)
      }
      sdml_ast::PrimitiveType::ShortStr | sdml_ast::PrimitiveType::LongStr => {
        string_field_def(&field.name)
      }
      sdml_ast::PrimitiveType::Boolean => boolean_field_def(&field.name),
      sdml_ast::PrimitiveType::DateTime => datetime_field_def(&field.name),
      sdml_ast::PrimitiveType::Int32 | sdml_ast::PrimitiveType::Int64 => {
        number_field_def(&field.name, NumberType::Integer)
      }
      sdml_ast::PrimitiveType::Float64 => {
        number_field_def(&field.name, NumberType::Float)
      }
    },
  }
}

/// Generates necessary filter arguments for id field.
fn id_field_def(
  field_name: &sdml_ast::Token,
) -> GraphQLGenResult<Vec<InputValueDefinition>> {
  string_field_def(&sdml_ast::Token::Ident(
    sdml_ast::Str::new(&open_crud_name::fields::Field::Id.common_name()),
    field_name.span(),
  ))
}

/// Generates necessary filter arguments for a string field.
fn string_field_def(
  field_name: &sdml_ast::Token,
) -> GraphQLGenResult<Vec<InputValueDefinition>> {
  let list_field_names_fmt = [("{}_in", "in list"), ("{}_not_in", "not in list")];
  let non_list_field_names_fmt = [
    ("{}", "equals"),
    ("{}_not", "not equals"),
    ("{}_contains", "contains substring"),
    ("{}_not_contains", "doesn't contain substring"),
    ("{}_starts_with", ""),
    ("{}_not_starts_with", ""),
    ("{}_ends_with", ""),
    ("{}_not_ends_with", ""),
    ("{}_lt", "less than"),
    ("{}_lte", "less than or equals"),
    ("{}_gt", "greater than"),
    ("{}_gte", "greater than or equals"),
  ];
  generate_where_input_filters(
    field_name,
    graphql_gen::FIELD_TYPE_NAME_STRING,
    &list_field_names_fmt,
    &non_list_field_names_fmt,
  )
}

enum NumberType {
  Integer,
  Float,
}
fn number_field_def(
  field_name: &sdml_ast::Token,
  number_type: NumberType,
) -> GraphQLGenResult<Vec<InputValueDefinition>> {
  // Names of the fields whose type is a list
  let list_field_names_fmt = [("{}_in", "in list"), ("{}_not_in", "not in list")];
  let non_list_field_names_fmt = [
    ("{}", "equals"),
    ("{}_not", "not equals"),
    ("{}_lt", "less than"),
    ("{}_lte", "less than or equals"),
    ("{}_gt", "greater than"),
    ("{}_gte", "greater than or equals"),
  ];

  let num_type = match number_type {
    NumberType::Integer => graphql_gen::FIELD_TYPE_NAME_INT,
    NumberType::Float => graphql_gen::FIELD_TYPE_NAME_FLOAT,
  };
  generate_where_input_filters(
    field_name,
    num_type,
    &list_field_names_fmt,
    &non_list_field_names_fmt,
  )
}

fn boolean_field_def(
  field_name: &sdml_ast::Token,
) -> GraphQLGenResult<Vec<InputValueDefinition>> {
  let non_list_field_names_fmt = [("{}", "equals"), ("{}_not", "not equals")];
  generate_where_input_filters(
    field_name,
    graphql_gen::FIELD_TYPE_NAME_BOOL,
    &[],
    &non_list_field_names_fmt,
  )
}

fn datetime_field_def(
  field_name: &sdml_ast::Token,
) -> GraphQLGenResult<Vec<InputValueDefinition>> {
  let list_field_names_fmt = [("{}_in", "in list"), ("{}_not_in", "not in list")];
  let non_list_field_names_fmt = [
    ("{}", "equals"),
    ("{}_not", "not equals"),
    ("{}_lt", "less than"),
    ("{}_lte", "less than or equals"),
    ("{}_gt", "greater than"),
    ("{}_gte", "greater than or equals"),
  ];
  generate_where_input_filters(
    field_name,
    graphql_gen::FIELD_TYPE_SCALAR_DATETIME,
    &list_field_names_fmt,
    &non_list_field_names_fmt,
  )
}

fn enum_field_def(
  field_name: &sdml_ast::Token,
  r#type: &sdml_ast::Type,
) -> GraphQLGenResult<Vec<InputValueDefinition>> {
  let enum_type_name = if let sdml_ast::Type::Enum { enum_ty_name } = r#type {
    enum_ty_name
      .try_get_ident_name()
      .map_err(ErrorGraphQLGen::new_sdml_error)
  } else {
    Err(ErrorGraphQLGen::SDMLError {
      error: format!("Incorrect type {:?} is passed instead of enum type", r#type),
      pos: r#type.token().span(),
    })
  }?;
  let list_field_names_fmt = [("{}_in", "in list"), ("{}_not_in", "not in list")];
  let non_list_field_names_fmt = [("{}", "equals"), ("{}_not", "not equals")];
  generate_where_input_filters(
    field_name,
    enum_type_name,
    &list_field_names_fmt,
    &non_list_field_names_fmt,
  )
}

fn relation_field_def(
  field_name: &sdml_ast::Token,
  target_relation: &sdml_ast::FieldType,
) -> GraphQLGenResult<Vec<InputValueDefinition>> {
  let field_name = field_name
    .try_get_ident_name()
    .map_err(ErrorGraphQLGen::new_sdml_error)?;
  let related_model_name = target_relation
    .r#type()
    .token()
    .try_get_ident_name()
    .map_err(ErrorGraphQLGen::new_sdml_error)?;
  let relation_where_filter =
    open_crud_name::types::FilterInput::Where.name(&related_model_name);
  // Many side of the relation
  if target_relation.is_array() {
    Ok(vec![
      InputValueDefinition {
        description: Some("condition must be true for all nodes".to_string()),
        name: Name::new(format!("{field_name}_every")),
        ty: Type::new(&relation_where_filter, TypeMod::Optional),
        default_value: None,
        directives: vec![],
      },
      InputValueDefinition {
        description: Some("condition must be true for at least 1 node".to_string()),
        name: Name::new(format!("{field_name}_some")),
        ty: Type::new(&relation_where_filter, TypeMod::Optional),
        default_value: None,
        directives: vec![],
      },
      InputValueDefinition {
        description: Some("condition must be false for all nodes".to_string()),
        name: Name::new(format!("{field_name}_none")),
        ty: Type::new(&relation_where_filter, TypeMod::Optional),
        default_value: None,
        directives: vec![],
      },
      InputValueDefinition {
        description: Some("is the relation field empty".to_string()),
        name: Name::new(format!("{field_name}_is_empty")),
        ty: Type::new(FIELD_TYPE_NAME_BOOL, TypeMod::Optional),
        default_value: None,
        directives: vec![],
      },
    ])
  } else {
    Ok(vec![
      InputValueDefinition {
        description: Some("condition must be true for related node".to_string()),
        name: Name::new(format!("{field_name}")),
        ty: Type::new(&relation_where_filter, TypeMod::Optional),
        default_value: None,
        directives: vec![],
      },
      InputValueDefinition {
        description: Some("is the relation field null".to_string()),
        name: Name::new(format!("{field_name}_is_null")),
        ty: Type::new(FIELD_TYPE_NAME_BOOL, TypeMod::Optional),
        default_value: None,
        directives: vec![],
      },
    ])
  }
}

/// Returns logical operation filters for where input type for the given model.
/// ### Arguments
/// * `model_name` - name of the model.
fn logical_operations_def(
  model_name: &sdml_ast::Token,
) -> GraphQLGenResult<Vec<InputValueDefinition>> {
  let model_name = model_name
    .try_get_ident_name()
    .map_err(ErrorGraphQLGen::new_sdml_error)?;
  let where_input_ty_name = open_crud_name::types::FilterInput::Where.name(model_name);
  Ok(vec![
    InputValueDefinition {
      description: Some("Logical AND on all given filters.".to_string()),
      name: Name::new("AND"),
      ty: Type::new_from_str(&format!("[{where_input_ty_name}!]")).unwrap(),
      default_value: None,
      directives: vec![],
    },
    InputValueDefinition {
      description: Some("Logical OR on all given filters.".to_string()),
      name: Name::new("OR"),
      ty: Type::new_from_str(&format!("[{where_input_ty_name}!]")).unwrap(),
      default_value: None,
      directives: vec![],
    },
    InputValueDefinition {
      description: Some("Logical NOT on all given filters combined by AND.".to_string()),
      name: Name::new("NOT"),
      ty: Type::new_from_str(&format!("[{where_input_ty_name}!]")).unwrap(),
      default_value: None,
      directives: vec![],
    },
  ])
}

/// Generates where input filters for the given field.
/// ### Arguments
///
/// * `field_name` - field name token from sdml ast.
/// * `field_type_name` - field's graphQL type name. Ex. "String"
/// * `list_field_names_fmt` -  array of input field names format, of type `list`.
/// It should be an array of tuple with 1st element being the field name, and 2nd element of tuple being its description.
/// Ex. \[("{}_in", "in list"), ("{}_not_in", "not in list")\]
/// * `non_list_field_names_fmt` - array of input field names format, of type `non-list`.
/// It should be an array of tuple with 1st element being the field name, and 2nd element of tuple being its description.
/// Ex. \[("{}", "equals"),("{}_not", "not equals")\]
#[inline]
fn generate_where_input_filters(
  field_name: &sdml_ast::Token,
  field_type_name: &str,
  list_field_names_fmt: &[(&str, &str)],
  non_list_field_names_fmt: &[(&str, &str)],
) -> GraphQLGenResult<Vec<InputValueDefinition>> {
  let field_name: &str = field_name
    .try_get_ident_name()
    .map_err(ErrorGraphQLGen::new_sdml_error)?;
  let non_list_fields =
    non_list_field_names_fmt
      .into_iter()
      .map(|(field_format, field_desc)| InputValueDefinition {
        description: Some(field_desc.to_string()),
        name: Name::new(field_format.replace("{}", field_name)),
        ty: Type::new_from_str(field_type_name).unwrap(),
        default_value: None,
        directives: vec![],
      });
  let list_fields = list_field_names_fmt
    .into_iter()
    .map(|(field_format, field_desc)| InputValueDefinition {
      description: Some(field_desc.to_string()),
      name: Name::new(field_format.replace("{}", field_name)),
      ty: Type::new_from_str(&format!("[{field_type_name}]")).unwrap(),
      default_value: None,
      directives: vec![],
    });

  Ok(non_list_fields.chain(list_fields).collect())
}

#[cfg(test)]
mod tests {
  use std::fs;

  use sdml_ast::{Span, Str};
  use sdml_parser;

  use super::*;

  #[test]
  fn test_where_input_def() {
    let mut expected_graphql_str = fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/input_type/user_where_input.graphql"
    ))
    .unwrap();
    expected_graphql_str.retain(|c| !c.is_whitespace());
    let sdml_str = fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/input_type/user_where_input.sdml"
    ))
    .unwrap();
    let data_model = sdml_parser::parse(&sdml_str)
      .expect("A valid SDML file shouldn't fail in parsing.");
    let user_model_sdml_ast = data_model
      .models()
      .get("User")
      .expect("User model should exist in the SDML.");
    let user_where_input_grapql_ast =
      where_input_def(user_model_sdml_ast).expect("It should return UserWhereInput");
    let mut user_where_input_graphql = user_where_input_grapql_ast.to_string();
    user_where_input_graphql.retain(|c| !c.is_whitespace());
    assert_eq!(expected_graphql_str, user_where_input_graphql)
  }

  #[test]
  fn test_input_filters_str_field_def() {
    let expected_str = r#"
"""equals"""
field: String
"""not equals"""
field_not: String
"""contains substring"""
field_contains: String
"""doesn't contain substring"""
field_not_contains: String
field_starts_with: String
field_not_starts_with: String
field_ends_with: String
field_not_ends_with: String
"""less than"""
field_lt: String
"""less than or equals"""
field_lte: String
"""greater than"""
field_gt: String
"""greater than or equals"""
field_gte: String
"""in list"""
field_in: [String]
"""not in list"""
field_not_in: [String]"#;
    let str_field_input_filters =
      string_field_def(&sdml_ast::Token::Ident(Str::new("field"), Span::new(0, 0)))
        .expect("It should be a valid output");
    let actual_str = str_field_input_filters
      .into_iter()
      .fold("".to_string(), |acc, x| format!("{}{}", acc, x.to_string()));
    assert_eq!(expected_str, actual_str);
  }

  #[test]
  fn test_input_filters_int_field_def() {
    let expected_str = r#"
"""equals"""
field: Int
"""not equals"""
field_not: Int
"""less than"""
field_lt: Int
"""less than or equals"""
field_lte: Int
"""greater than"""
field_gt: Int
"""greater than or equals"""
field_gte: Int
"""in list"""
field_in: [Int]
"""not in list"""
field_not_in: [Int]"#;
    let int_field_input_filters = number_field_def(
      &sdml_ast::Token::Ident(Str::new("field"), Span::new(0, 0)),
      NumberType::Integer,
    )
    .expect("It should be a valid output");
    let actual_str = int_field_input_filters
      .into_iter()
      .fold("".to_string(), |acc, x| format!("{}{}", acc, x.to_string()));
    assert_eq!(expected_str, actual_str);
  }

  #[test]
  fn test_input_filters_float_field_def() {
    let expected_str = r#"
"""equals"""
field: Float
"""not equals"""
field_not: Float
"""less than"""
field_lt: Float
"""less than or equals"""
field_lte: Float
"""greater than"""
field_gt: Float
"""greater than or equals"""
field_gte: Float
"""in list"""
field_in: [Float]
"""not in list"""
field_not_in: [Float]"#;
    let float_field_input_filters = number_field_def(
      &sdml_ast::Token::Ident(Str::new("field"), Span::new(0, 0)),
      NumberType::Float,
    )
    .expect("It should be a valid output");
    let actual_str = float_field_input_filters
      .into_iter()
      .fold("".to_string(), |acc, x| format!("{}{}", acc, x.to_string()));
    assert_eq!(expected_str, actual_str);
  }

  #[test]
  fn test_input_filters_boolean_field_def() {
    let expected_str = r#"
"""equals"""
field: Boolean
"""not equals"""
field_not: Boolean"#;
    let boolean_field_input_filters =
      boolean_field_def(&sdml_ast::Token::Ident(Str::new("field"), Span::new(0, 0)))
        .expect("It should be a valid output");
    let actual_str = boolean_field_input_filters
      .into_iter()
      .fold("".to_string(), |acc, x| format!("{}{}", acc, x.to_string()));
    assert_eq!(expected_str, actual_str);
  }

  #[test]
  fn test_input_filters_enum_field_def() {
    let expected_str = r#"
"""equals"""
userRole: Role
"""not equals"""
userRole_not: Role
"""in list"""
userRole_in: [Role]
"""not in list"""
userRole_not_in: [Role]"#;
    let enum_field_input_filters = enum_field_def(
      &sdml_ast::Token::Ident(Str::new("userRole"), Span::new(0, 0)),
      &sdml_ast::Type::Enum {
        enum_ty_name: sdml_ast::Token::Ident(Str::new("Role"), Span::new(0, 0)),
      },
    )
    .expect("It should be a valid output");
    let actual_str = enum_field_input_filters
      .into_iter()
      .fold("".to_string(), |acc, x| format!("{}{}", acc, x.to_string()));
    assert_eq!(expected_str, actual_str);
  }

  #[test]
  fn test_input_logical_operations_def() {
    let expected_str = r#"
"""Logical AND on all given filters."""
AND: [UserWhereInput!]
"""Logical OR on all given filters."""
OR: [UserWhereInput!]
"""Logical NOT on all given filters combined by AND."""
NOT: [UserWhereInput!]"#;
    let logical_operations =
      logical_operations_def(&sdml_ast::Token::Ident(Str::new("User"), Span::new(0, 0)))
        .expect("It should be a valid output");
    let actual_str = logical_operations
      .into_iter()
      .fold("".to_string(), |acc, x| format!("{acc}{x}"));
    println!("{}", actual_str);
    assert_eq!(expected_str, actual_str);
  }
}
