//! Generates the root level mutation type
use input_type::update;

use super::*;

/// Code-gen root Mutation type with required OpenCRUD fields
/// to Create, Update / Upsert, Delete for all models.
/// ### Arguments.
///  * `models` - array of models in sdml.
/// ### Returns.
/// Root level mutation type definition.
pub(in crate::graphql_gen) fn root_mutation_type_def<'src>(
    models: &Vec<&sdml_ast::ModelDecl<'src>>,
) -> GraphQLGenResult<TypeDefinition> {
    let fields = models.iter().try_fold(Vec::new(), |mut acc, model| {
        acc.extend(root_mutation_fields(model)?);
        Ok(acc)
    })?;
    Ok(TypeDefinition {
        extend: false,
        description: None,
        directives: vec![],
        name: open_crud_name::types::MutationType::RootMutation.common_name(),
        kind: TypeKind::Object(ObjectType {
            implements: vec![],
            fields,
        }),
    })
}

fn root_mutation_fields<'src>(
    model: &sdml_ast::ModelDecl<'src>,
) -> GraphQLGenResult<Vec<FieldDefinition>> {
    let model_name = model.name
        .try_get_ident_name()
        .map_err(ErrorGraphQLGen::new_sdml_error)?;
    let mut fields = vec![
        // Create
        FieldDefinition {
            description: Some(format!("Returns newly created '{model_name}' object if successful.")),
            name: open_crud_name::fields::MutationType::Create.name(model_name),
            arguments: vec![InputValueDefinition {
                description: None,
                name: open_crud_name::fields::MutationInputArg::Data.common_name(),
                ty: open_crud_name::types::CreateInput::Create
                    .ty(model_name, TypeMod::NonOptional),
                default_value: None,
                directives: vec![],
            }],
            ty: Type::new(model_name, TypeMod::Optional),
            directives: vec![],
        },
        // Update
        FieldDefinition {
            description: Some(format!("Returns the updated '{model_name}' object if successful.")),
            name: open_crud_name::fields::MutationType::Update.name(model_name),
            arguments: vec![
                // where
                InputValueDefinition {
                    description: None,
                    name: open_crud_name::fields::MutationInputArg::Where.common_name(),
                    ty: open_crud_name::types::FilterInput::WhereUnique
                        .ty(model_name, TypeMod::NonOptional),
                    default_value: None,
                    directives: vec![],
                },
                // data
                InputValueDefinition {
                    description: None,
                    name: open_crud_name::fields::MutationInputArg::Data.common_name(),
                    ty: open_crud_name::types::UpdateInput::Update
                        .ty(model_name, TypeMod::NonOptional),
                    default_value: None,
                    directives: vec![],
                },
            ],
            ty: Type::new(model_name, TypeMod::Optional),
            directives: vec![],
        },
        // Delete
        FieldDefinition {
            description: Some(format!("Returns the deleted '{model_name}' object if successful.")),
            name: open_crud_name::fields::MutationType::Delete.name(model_name),
            arguments: vec![
                // where
                InputValueDefinition {
                    description: None,
                    name: open_crud_name::fields::MutationInputArg::Where.common_name(),
                    ty: open_crud_name::types::FilterInput::WhereUnique
                        .ty(model_name, TypeMod::NonOptional),
                    default_value: None,
                    directives: vec![],
                },
            ],
            ty: Type::new(model_name, TypeMod::Optional),
            directives: vec![],
        },
        // Upsert
        FieldDefinition {
            description: Some(format!("Returns the upserted (either created new or updated) '{model_name}' object if successful.")),
            name: open_crud_name::fields::MutationType::Upsert.name(model_name),
            arguments: vec![
                // where
                InputValueDefinition {
                    description: None,
                    name: open_crud_name::fields::MutationInputArg::Where.common_name(),
                    ty: open_crud_name::types::FilterInput::WhereUnique
                        .ty(model_name, TypeMod::NonOptional),
                    default_value: None,
                    directives: vec![],
                },
                // data
                InputValueDefinition {
                    description: None,
                    name: open_crud_name::fields::MutationInputArg::Data.common_name(),
                    ty: open_crud_name::types::UpdateInput::Upsert
                        .ty(model_name, TypeMod::NonOptional),
                    default_value: None,
                    directives: vec![],
                },
            ],
            ty: Type::new(model_name, TypeMod::Optional),
            directives: vec![],
        },
        
        // DeleteMany
        FieldDefinition {
            description: Some(format!("Returns the deleted '{model_name}' objects.")),
            name: open_crud_name::fields::MutationType::DeleteMany.name(model_name),
            arguments: vec![
                // where
                InputValueDefinition {
                    description: None,
                    name: open_crud_name::fields::MutationInputArg::Where.common_name(),
                    ty: open_crud_name::types::FilterInput::Where.ty(model_name, TypeMod::NonOptional),
                    default_value: None,
                    directives: vec![],
                },
                // skip
                InputValueDefinition {
                    description: None,
                    name: open_crud_name::fields::MutationInputArg::Skip.common_name(),
                    ty: Type::new(FIELD_TYPE_NAME_INT, TypeMod::Optional),
                    default_value: None,
                    directives: vec![],
                },
                // after
                InputValueDefinition {
                    description: None,
                    name: open_crud_name::fields::MutationInputArg::After.common_name(),
                    ty: open_crud_name::types::OpenCRUDType::IdType.common_ty(TypeMod::Optional),
                    default_value: None,
                    directives: vec![],
                },
                // before
                InputValueDefinition {
                    description: None,
                    name: open_crud_name::fields::MutationInputArg::Before.common_name(),
                    ty: open_crud_name::types::OpenCRUDType::IdType.common_ty(TypeMod::Optional),
                    default_value: None,
                    directives: vec![],
                },
                // first
                InputValueDefinition {
                    description: None,
                    name: open_crud_name::fields::MutationInputArg::First.common_name(),
                    ty: Type::new(FIELD_TYPE_NAME_INT, TypeMod::Optional),
                    default_value: None,
                    directives: vec![],
                },
                // last
                InputValueDefinition {
                    description: None,
                    name: open_crud_name::fields::MutationInputArg::Last.common_name(),
                    ty: Type::new(FIELD_TYPE_NAME_INT, TypeMod::Optional),
                    default_value: None,
                    directives: vec![],
                },
            ],
            ty: open_crud_name::types::AuxiliaryType::Connection.ty(model_name, TypeMod::NonOptional),
            directives: vec![]
        }
    ];

    // check if we need UpdateMany API.
    if update::has_update_many_input(model) {
        // updateMany
        fields.push(
            FieldDefinition {
                description: Some(format!("Returns the updated '{model_name}' objects.")),
                name: open_crud_name::fields::MutationType::UpdateMany.name(model_name),
                arguments: vec![
                    // where
                    InputValueDefinition {
                        description: None,
                        name: open_crud_name::fields::MutationInputArg::Where.common_name(),
                        ty: open_crud_name::types::FilterInput::Where.ty(model_name, TypeMod::NonOptional),
                        default_value: None,
                        directives: vec![],
                    },
                    // data
                    InputValueDefinition {
                        description: None,
                        name: open_crud_name::fields::MutationInputArg::Data.common_name(),
                        ty: open_crud_name::types::UpdateInput::UpdateMany
                            .ty(model_name, TypeMod::NonOptional),
                        default_value: None,
                        directives: vec![],
                    },
                    // skip
                    InputValueDefinition {
                        description: None,
                        name: open_crud_name::fields::MutationInputArg::Skip.common_name(),
                        ty: Type::new(FIELD_TYPE_NAME_INT, TypeMod::Optional),
                        default_value: None,
                        directives: vec![],
                    },
                    // after
                    InputValueDefinition {
                        description: None,
                        name: open_crud_name::fields::MutationInputArg::After.common_name(),
                        ty: open_crud_name::types::OpenCRUDType::IdType.common_ty(TypeMod::Optional),
                        default_value: None,
                        directives: vec![],
                    },
                    // before
                    InputValueDefinition {
                        description: None,
                        name: open_crud_name::fields::MutationInputArg::Before.common_name(),
                        ty: open_crud_name::types::OpenCRUDType::IdType.common_ty(TypeMod::Optional),
                        default_value: None,
                        directives: vec![],
                    },
                    // first
                    InputValueDefinition {
                        description: None,
                        name: open_crud_name::fields::MutationInputArg::First.common_name(),
                        ty: Type::new(FIELD_TYPE_NAME_INT, TypeMod::Optional),
                        default_value: None,
                        directives: vec![],
                    },
                    // last
                    InputValueDefinition {
                        description: None,
                        name: open_crud_name::fields::MutationInputArg::Last.common_name(),
                        ty: Type::new(FIELD_TYPE_NAME_INT, TypeMod::Optional),
                        default_value: None,
                        directives: vec![],
                    },
                ],
                ty: open_crud_name::types::AuxiliaryType::Connection.ty(model_name, TypeMod::NonOptional),
                directives: vec![]
            }
        );
    }

    Ok(fields)
}

#[cfg(test)]
mod tests {
    use chumsky::prelude::*;
    use sdml_parser::parser;
    use std::fs;

    #[test]
    fn test_root_mutation_type_def() {
        let sdml_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/test_root_mutation_type_def.sdml"
        ))
        .unwrap();
        let mut expected_graphql_str = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/test_root_mutation_type_def.graphql"
        ))
        .unwrap();
        expected_graphql_str.retain(|c| !c.is_whitespace());

        let sdml_ast = parser::delcarations()
            .parse(&sdml_str)
            .into_result()
            .expect("It should be a valid sdml file");
        let sdml_ast = parser::semantic_analysis(sdml_ast)
            .expect("Semantic analysis should succeed!");
        let root_query_type =
            super::root_mutation_type_def(&sdml_ast.models_sorted()).unwrap();
        let mut actual_graphql_str = root_query_type.to_string();
        actual_graphql_str.retain(|c| !c.is_whitespace());
        assert_eq!(expected_graphql_str, actual_graphql_str);
    }
}
