//! Impelments code-gen functions for generating WhereUniqueInput filter type.
use super::*;

/// Generates WhereUniqueInput filter type for the given model.
/// When this fitler gets passed as an argument,
/// it will exactly match *at-most* 1 record in the graphQL response.
pub fn where_unique_type_def<'src>(
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
        .collect::<Result<Vec<InputValueDefinition>, ErrorGraphQLGen>>()?;
    let model_name = model
        .name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?; // Note: Model should have a valid identifier, otherwise it will be caught by SDML parser! So we can just use unwrap()..
    Ok(TypeDefinition {
        extend: false,
        description: Some("The where unique filter which can match at-most 1 object.".to_string()),
        name: Name::new(FilterType::WhereUniqueInput.name(model_name)),
        directives: vec![],
        kind: TypeKind::InputObject(InputObjectType {
            fields: unique_field_filters,
        }),
    })
}

fn unique_scalar_field_to_filter<'src>(
    field: &sdml_ast::FieldDecl<'src>,
) -> GraphQLGenResult<InputValueDefinition> {
    let is_id_field = field.has_id_attrib();
    let ty = match &*field.field_type.r#type() {
        sdml_ast::Type::Enum { enum_ty_name } => {
            let ty_name = enum_ty_name
                .try_get_ident_name()
                .map_err(ErrorGraphQLGen::new_sdml_error)?;
            Ok(Type::new(ty_name).unwrap())
        }
        sdml_ast::Type::Primitive {
            r#type: primitive_type,
            ..
        } => match primitive_type {
            sdml_ast::PrimitiveType::ShortStr if is_id_field => {
                Ok(Type::new(ID_TYPE_NAME).unwrap())
            }
            sdml_type => Ok(Type::new_from(sdml_type)),
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
            Name::new(ID_FIELD_NAME)
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
