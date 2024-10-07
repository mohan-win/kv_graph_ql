use super::*;

/// Input type used to create a new object.
/// Ex. UserCreateInput creates a new user.
pub(in crate::graphql_gen) fn create_input_def<'src>(
    model: &sdml_ast::ModelDecl<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    let mut non_relation_fields = vec![];
    let mut relation_fields = vec![];
    let mut relation_scalar_field_names = vec![];
    model
        .fields
        .iter()
        .for_each(|field| match &*field.field_type.r#type() {
            sdml_ast::Type::Unknown(..) => {
                panic!("Can't transpile field with unknown type to graphql")
            }
            sdml_ast::Type::Relation(edge) => {
                relation_fields.push(field);
                edge.scalar_field_name().map(|fld_name| {
                    relation_scalar_field_names.push(fld_name.ident_name())
                });
            }
            sdml_ast::Type::Primitive { .. } | sdml_ast::Type::Enum { .. } => {
                non_relation_fields.push(field);
            }
        });
    let non_realtion_fields = non_relation_fields.into_iter().filter(|field| {
        // Note: Filter out relation_scalar fields & auto generated ids.
        // Why?
        // 1. Relation scalar fields will be populated with the content of *CreateInlineInput fields.
        // 2. Autogenerated ids will be auto-generated by DB engine module, not inputted by the user.
        !relation_scalar_field_names.contains(&field.name.ident_name())
            && !field.is_auto_gen_id()
    });

    let mut input_field_defs = non_realtion_fields
        .map(non_relation_field_input_def)
        .collect::<GraphQLGenResult<Vec<InputValueDefinition>>>(
    )?;
    let relation_input_field_defs = relation_fields
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
        name: open_crud_name::CreateInputType::CreateInput.name(model_name),
        directives: vec![],
        kind: TypeKind::InputObject(InputObjectType {
            fields: input_field_defs,
        }),
    })
}

/// Code-gen input arg for the non-relation field.
/// **Note:**
/// The auto-generated id field won't be represented in CreateInput type,
/// so this function will return None for it.
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
    let mut description = None;
    let default_attribute = field.default_attribute();
    let type_mod = if default_attribute.is_some() {
        description = Some(format!(
            "Default value '{}' will be assigned if no value is passed to this input arg.",
            default_attribute.unwrap().arg.as_ref().unwrap()
        ));
        // If the field has default attribute, make the type as optional.
        // Note: The default attribute is allowed only on a scalar field.
        // Otherwise SDML parser would fail. Hence we need not bother
        // about @default(..) being present on an array model field.
        sdml_ast::FieldTypeMod::Optional
    } else {
        field.field_type.type_mod
    };

    Ok(InputValueDefinition {
        description,
        name: field
            .name
            .try_get_graphql_name()
            .map_err(ErrorGraphQLGen::new_sdml_error)?,
        ty: Type::new(&ty_str, type_mod),
        default_value: None, // Note: The default value is set to None. The appropriate default
        directives: vec![],
    })
}

/// Code-gen input arg for the relation field.
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
            open_crud_name::CreateInputType::CreateManyInlineInput
                .ty(referenced_model_name, sdml_ast::FieldTypeMod::NonOptional)
        } else if field.field_type.is_optional() {
            open_crud_name::CreateInputType::CreateOneInlineInput
                .ty(referenced_model_name, sdml_ast::FieldTypeMod::Optional)
        } else {
            open_crud_name::CreateInputType::CreateOneInlineInput
                .ty(referenced_model_name, sdml_ast::FieldTypeMod::NonOptional)
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

/// Input type used to create one object in one side of the relation
/// in a nested create.
/// Ex. ProfileCreateOneInlineInput will be used inside UserCreateInput
/// to create user profile inline when creating a new user.
pub(in crate::graphql_gen) fn create_one_inline_input_def<'src>(
    model: &sdml_ast::ModelDecl<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    unimplemented!()
}

/// Input type used to create the many objects in a relation
/// in a nested create.
/// Ex. PostCreateManyInlineInput will be used inside UserCreateInput
/// to create posts inline when creating a new user.
pub(in crate::graphql_gen) fn create_many_inline_input_def<'src>(
    model: &sdml_ast::ModelDecl<'src>,
) -> GraphQLGenResult<TypeDefinition> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::create_input_def;

    use chumsky::prelude::*;
    use sdml_parser::parser;
    use std::fs;

    #[test]
    fn test_user_create_input_def() {
        let mut expected_graphql_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/test_create_user_input_def.graphql"
        ))
        .unwrap();
        expected_graphql_str.retain(|c| !c.is_whitespace());

        let sdml_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/test_create_user_input_def.sdml"
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
        let create_user_input_type_graphql_ast =
            create_input_def(user_model_sdml_ast).expect("It should return User");

        let mut create_user_input_type_graphql_str =
            create_user_input_type_graphql_ast.to_string();
        eprintln!("{}", create_user_input_type_graphql_str);

        create_user_input_type_graphql_str.retain(|c| !c.is_whitespace());
        assert_eq!(expected_graphql_str, create_user_input_type_graphql_str)
    }
}
