//! Module to code-gen required GraphQL object type for the given SDML model type.
use super::*;
use aux_type::connection_types_def;

/// Code-gen GraphQL type and its auxiliary types for the given model.
pub fn type_and_aux_types_def<'src>(
    model: &sdml_ast::ModelDecl<'src>,
    pg_info: &TypeDefinition,
    aggregate: &TypeDefinition,
) -> GraphQLGenResult<Vec<TypeDefinition>> {
    let mut result = vec![];
    result.push(type_def(model)?);
    result.extend(connection_types_def(&model.name, pg_info, aggregate)?);
    Ok(result)
}

fn type_def<'src>(model: &sdml_ast::ModelDecl<'src>) -> GraphQLGenResult<TypeDefinition> {
    let model_name = model
        .name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    let fields = model
        .fields
        .iter()
        .try_fold(Vec::new(), |mut acc, model_fld| {
            acc.extend(field_def(model, model_fld)?);
            Ok(acc)
        })?;

    Ok(TypeDefinition {
        extend: false,
        description: Some(model_name.to_string()),
        name: Name::new(model_name),
        directives: vec![],
        kind: TypeKind::Object(ObjectType {
            implements: vec![Name::new(INTERFACE_NODE_NAME)],
            fields,
        }),
    })
}

#[inline(always)]
fn field_def<'src>(
    model: &sdml_ast::ModelDecl<'src>,
    field: &sdml_ast::FieldDecl<'src>,
) -> GraphQLGenResult<Vec<FieldDefinition>> {
    match &*field.field_type.r#type() {
        sdml_ast::Type::Unknown(..) => panic!("Invalid field type!"),
        sdml_ast::Type::Relation(..) => relation_field_def(model, field),
        _ => Ok(vec![non_relation_field_def(field)?]),
    }
}

/// Code-gen for non-relation field.
fn non_relation_field_def<'src>(
    field: &sdml_ast::FieldDecl<'src>,
) -> GraphQLGenResult<FieldDefinition> {
    debug_assert!(
        match &*field.field_type.r#type() {
            sdml_ast::Type::Relation(..) | sdml_ast::Type::Unknown(..) => false,
            _ => true,
        },
        "Only non-relation field is allowed here!"
    );

    let ty_name_str = match &*field.field_type.r#type() {
        sdml_ast::Type::Primitive { r#type, .. } => {
            Ok(Type::map_sdml_type_to_graphql_ty_name(r#type))
        }
        sdml_ast::Type::Enum { enum_ty_name } => Ok(enum_ty_name
            .ident_name()
            .expect("Enum name should be a valid identifier")
            .to_string()),
        _ => Err(ErrorGraphQLGen::SDMLError {
            error: "Non-relational field should be either primitive type or enum"
                .to_string(),
            pos: field.name.span(),
        }),
    }?;

    let mut field_name = field
        .name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    let mut directives = vec![];
    if field.has_id_attrib() {
        directives.push(ConstDirective {
            name: Name::new("map"),
            arguments: vec![(
                Name::new("name"),
                ConstValue::String(field_name.to_string()),
            )],
        });
        field_name = FIELD_NAME_ID; // Note:Rename the field to "id".
        directives.push(ConstDirective {
            name: Name::new("unique"),
            arguments: vec![],
        });
    } else if field.has_unique_attrib() {
        directives.push(ConstDirective {
            name: Name::new("unique"),
            arguments: vec![],
        });
    }

    Ok(FieldDefinition {
        description: None,
        name: Name::new(field_name),
        arguments: vec![],
        ty: Type::new(&ty_name_str, field.field_type.type_mod),
        directives,
    })
}

/// Returns field arguments for the `relation` array field.
fn array_field_args<'src>(
    referenced_model_name: &'src str,
) -> GraphQLGenResult<Vec<InputValueDefinition>> {
    let mut args = vec![];
    args.push(InputValueDefinition {
        description: None,
        name: Name::new(FIELD_ARG_WHERE),
        ty: open_crud::FilterType::WhereInput
            .ty(referenced_model_name, sdml_ast::FieldTypeMod::Optional),
        default_value: None,
        directives: vec![],
    });
    args.push(InputValueDefinition {
        description: None,
        name: Name::new(FIELD_ARG_ORDER_BY),
        ty: open_crud::InputType::OrderBy
            .ty(referenced_model_name, sdml_ast::FieldTypeMod::Optional),
        default_value: None,
        directives: vec![],
    });
    args.push(InputValueDefinition {
        description: None,
        name: Name::new(FIELD_ARG_SKIP),
        ty: Type::new(FIELD_TYPE_NAME_INT, sdml_ast::FieldTypeMod::Optional),
        default_value: None,
        directives: vec![],
    });
    args.push(InputValueDefinition {
        description: None,
        name: Name::new(FIELD_ARG_AFTER),
        ty: Type::new(FIELD_TYPE_NAME_ID, sdml_ast::FieldTypeMod::Optional),
        default_value: None,
        directives: vec![],
    });
    args.push(InputValueDefinition {
        description: None,
        name: Name::new(FIELD_ARG_BEFORE),
        ty: Type::new(FIELD_TYPE_NAME_ID, sdml_ast::FieldTypeMod::Optional),
        default_value: None,
        directives: vec![],
    });
    args.push(InputValueDefinition {
        description: None,
        name: Name::new(FIELD_ARG_FIRST),
        ty: Type::new(FIELD_TYPE_NAME_INT, sdml_ast::FieldTypeMod::Optional),
        default_value: None,
        directives: vec![],
    });
    args.push(InputValueDefinition {
        description: None,
        name: Name::new(FIELD_ARG_LAST),
        ty: Type::new(FIELD_TYPE_NAME_INT, sdml_ast::FieldTypeMod::Optional),
        default_value: None,
        directives: vec![],
    });
    Ok(args)
}

/// Code-gen for relation field.
fn relation_field_def<'src>(
    model: &sdml_ast::ModelDecl<'src>,
    field: &sdml_ast::FieldDecl<'src>,
) -> GraphQLGenResult<Vec<FieldDefinition>> {
    let field_type = field.field_type.r#type();
    let relation_edge = match &*field_type {
        sdml_ast::Type::Relation(edge) => edge,
        _ => panic!("Only relation field is allowed here!"),
    };

    let field_name = field
        .name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    let referenced_model_name = relation_edge
        .referenced_model_name()
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    let field_type = Type::new(referenced_model_name, field.field_type.type_mod);

    if field.field_type.is_array() {
        Ok(vec![
            FieldDefinition {
                description: None,
                name: Name::new(field_name),
                arguments: array_field_args(referenced_model_name)?,
                ty: field_type,
                directives: vec![],
            },
            FieldDefinition {
                description: None,
                name: Name::new(format!("{field_name}Connection")),
                arguments: array_field_args(referenced_model_name)?,
                ty: open_crud::AuxiliaryType::Connection
                    .ty(referenced_model_name, field.field_type.type_mod),
                directives: vec![],
            },
        ])
    } else {
        Ok(vec![FieldDefinition {
            description: None,
            name: Name::new(field_name),
            arguments: vec![],
            ty: field_type,
            directives: vec![],
        }])
    }
}

#[cfg(test)]
mod tests {
    use chumsky::prelude::*;
    use sdml_parser::parser;
    use std::fs;

    use crate::graphql_gen::{
        aux_type,
        r#type::{type_and_aux_types_def, type_def},
    };

    #[test]
    fn test_type_def() {
        let mut expected_graphql_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/test_type_def.graphql"
        ))
        .unwrap();
        expected_graphql_str.retain(|c| !c.is_whitespace());
        let sdml_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/test_type_def.sdml"
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
        let user_type_graphql_ast =
            type_def(user_model_sdml_ast).expect("It should return User");

        let mut user_type_graphql = user_type_graphql_ast.to_string();
        user_type_graphql.retain(|c| !c.is_whitespace());
        assert_eq!(expected_graphql_str, user_type_graphql)
    }

    #[test]
    fn test_type_and_aux_types_def() {
        let mut expected_graphql_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/test_type_and_aux_types_def.graphql"
        ))
        .unwrap();
        expected_graphql_str.retain(|c| !c.is_whitespace());
        let sdml_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/test_type_and_aux_types_def.sdml"
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
        let pg_info =
            aux_type::page_info_type_def().expect("pg_info can't fail to generate");
        let aggregage =
            aux_type::aggregage_type_def().expect("aggregate can't fail to generate");
        let user_types_graphql_ast =
            type_and_aux_types_def(user_model_sdml_ast, &pg_info, &aggregage)
                .expect("It should return User and their aux types!");

        let mut user_types_graphql = user_types_graphql_ast
            .into_iter()
            .fold("".to_string(), |acc, ty| {
                format!("{}{}", acc, ty.to_string())
            });
        user_types_graphql.retain(|c| !c.is_whitespace());
        assert_eq!(expected_graphql_str, user_types_graphql)
    }
}
