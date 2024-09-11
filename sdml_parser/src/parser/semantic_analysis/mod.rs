use crate::ast::{DataModel, Declaration, EnumDecl, FieldDecl, ModelDecl, Span, Token, Type};
use std::{
    collections::{HashMap, HashSet},
    ops::{ControlFlow, Deref},
};

/// Error Module
pub mod err;
use attribute::ATTRIB_NAME_ID;
use err::SemanticError;
use relation::RelationMap;

/// Module for attribute related semantic analysis and validation.
mod attribute;

/// Module for relation related semantic analysis and validation.
mod relation;

/// This function segregates the declartions into configs, enums & models
/// # params
/// `check_duplicate_types` - flag to check for duplicate types.
/// # returns
/// - segregated data_model
/// - (or) SemanticError::DuplicateTypeDefinition if duplicate types types found.
pub(crate) fn to_data_model<'src>(
    delcarations: Vec<Declaration<'src>>,
    check_duplicate_types: bool,
) -> Result<DataModel<'src>, Vec<SemanticError<'src>>> {
    let mut errs: Vec<SemanticError<'src>> = Vec::new();
    let mut type_set: HashSet<(&'src str, Span)> = HashSet::new();

    let mut data_model = DataModel::new();

    for decl in delcarations.into_iter() {
        let (type_name, span) = match decl {
            Declaration::Config(c) => {
                let type_name = c.name.ident_name().unwrap();
                let span = c.name.span();
                data_model.configs_mut().insert(type_name, c);
                (type_name, span)
            }
            Declaration::Enum(e) => {
                let type_name = e.name.ident_name().unwrap();
                let span = e.name.span();
                data_model
                    .enums_mut()
                    .insert(e.name.ident_name().unwrap(), e);
                (type_name, span)
            }
            Declaration::Model(m) => {
                let type_name = m.name.ident_name().unwrap();
                let span = m.name.span();
                data_model
                    .models_mut()
                    .insert(m.name.ident_name().unwrap(), m);
                (type_name, span)
            }
        };

        // error for duplicate types.
        if check_duplicate_types {
            let type_exists = type_set.iter().try_for_each(|(name, _span)| {
                if name.eq(&type_name) {
                    ControlFlow::Break(true)
                } else {
                    ControlFlow::Continue(())
                }
            });

            if let ControlFlow::Break(true) = type_exists {
                errs.push(SemanticError::TypeDuplicateDefinition { span, type_name });
            } else {
                type_set.insert((type_name, span));
            }
        }
    }
    if errs.len() > 1 {
        Err(errs)
    } else {
        Ok(data_model)
    }
}

/// This function semantically updates the AST to,
/// a. To identify the actual type of user defined types (Enum / Model) of a field. In the inital pass the user defined field_types are not determined.
/// b. Surface known semantic errors.
///
/// # Returns
/// - () if there are no errors during update.
/// - or array of errors if known semantic errors found.
pub(crate) fn semantic_update<'src>(
    input_ast: &mut DataModel<'src>,
) -> Result<(), Vec<SemanticError<'src>>> {
    let mut relations = RelationMap::new();
    let mut errs: Vec<SemanticError<'src>> = Vec::new();
    input_ast.models().values().for_each(|model| {
        // Make sure each model has a valid Id field.
        if let Err(err) = validate_model_id_field(model) {
            errs.push(err)
        }
        model.fields.iter().for_each(|field| {
            match get_actual_type(model, field, input_ast.models(), input_ast.enums()) {
                Ok(actual_type) => {
                    actual_type.map(|actual_type| {
                        field.field_type.set_type(actual_type);
                        if let Type::Relation(edge) = field.field_type.r#type().deref() {
                            let _ = relations
                                .add_relation_edge(edge.clone(), &field.name, &model.name) // Clone!
                                .map_err(|err| errs.push(err));
                        }
                    });
                }
                Err(err) => errs.push(err),
            };
            let _ = attribute::validate_attributes(field, model, input_ast.enums())
                .map_err(|err| errs.push(err));
        });
    });
    relations.get_valid_relations().map_or_else(
        |rel_errs| errs.extend(rel_errs.into_iter()),
        |valid_relations| {
            input_ast
                .relations_mut()
                .extend(valid_relations.into_iter())
        },
    );
    if errs.len() > 0 {
        Err(errs)
    } else {
        Ok(())
    }
}

/// This function finds and returns the actual type for the field if it's type is Unknown
/// If the field type is already known, it returns `None`.
/// But if its unable to locate the Uknown type, then it returns SemanticError::UndefinedType.
fn get_actual_type<'src>(
    model: &ModelDecl<'src>,
    field: &FieldDecl<'src>,
    models: &HashMap<&'src str, ModelDecl<'src>>,
    enums: &HashMap<&'src str, EnumDecl<'src>>,
) -> Result<Option<Type<'src>>, SemanticError<'src>> {
    if let Type::Unknown(type_name_tok) = &*field.field_type.r#type() {
        let type_name = type_name_tok.ident_name().unwrap();
        match models.get(type_name) {
            Some(referenced_model) => Ok(Some(Type::Relation(relation::get_relation_edge(
                model,
                field,
                referenced_model,
            )?))),
            None => match enums.get(type_name) {
                Some(_) => Ok(Some(Type::Enum {
                    enum_ty_name: type_name_tok.clone(), // Clone
                })),
                None => Err(SemanticError::TypeUndefined {
                    span: type_name_tok.span(),
                    type_name: type_name_tok.ident_name().unwrap(),
                    field_name: field.name.ident_name().unwrap(),
                    model_name: model.name.ident_name().unwrap(),
                }),
            },
        }
    } else {
        Ok(None)
    }
}

/// Make sure model has ONLY one field marked with @id
fn validate_model_id_field<'src>(model: &ModelDecl<'src>) -> Result<(), SemanticError<'src>> {
    let mut id_fields = model.fields.iter().filter(|field| {
        field
            .attributes
            .iter()
            .find(|attrib| {
                if let Token::Ident(ATTRIB_NAME_ID, _) = attrib.name {
                    true
                } else {
                    false
                }
            })
            .is_some()
    });

    let id_field = id_fields.next();
    if id_field.is_none() {
        return Err(SemanticError::ModelIdFieldMissing {
            span: model.name.span(),
            model_name: model.name.ident_name().unwrap(),
        });
    } else if let Some(second_id_field) = id_fields.next() {
        // Is there more than one Id field in a Model ?
        return Err(SemanticError::ModelIdFieldDuplicate {
            span: second_id_field.name.span(),
            field_name: second_id_field.name.ident_name().unwrap(),
            model_name: model.name.ident_name().unwrap(),
        });
    }

    if id_field.is_some_and(|id_field| id_field.field_type.is_optional) {
        // Is id field is an optional one ?
        Err(SemanticError::ModelIdFieldOptional {
            span: id_field.unwrap().name.span(),
            field_name: id_field.unwrap().name.ident_name().unwrap(),
            model_name: model.name.ident_name().unwrap(),
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::Span;

    use super::*;
    use chumsky::prelude::*;

    #[test]
    fn test_duplicate_types() {
        let duplicate_types_sdml = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/semantic_analysis/duplicate_types.sdml"
        ))
        .unwrap();

        let decls = crate::parser::delcarations()
            .parse(&duplicate_types_sdml)
            .into_result()
            .unwrap();
        match to_data_model(decls, true) {
            Ok(_) => assert!(false, "Model file with duplicate types should throw err!"),
            Err(errs) => {
                assert_eq!(
                    errs,
                    vec![
                        SemanticError::TypeDuplicateDefinition {
                            span: Span::new(52, 54),
                            type_name: "db"
                        },
                        SemanticError::TypeDuplicateDefinition {
                            span: Span::new(294, 311),
                            type_name: "User"
                        },
                        SemanticError::TypeDuplicateDefinition {
                            span: Span::new(666, 670),
                            type_name: "Role"
                        }
                    ]
                )
            }
        }
    }

    #[test]
    fn test_semantic_update() {
        let semantic_errs_sdml = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/semantic_analysis/semantic_errs.sdml"
        ))
        .unwrap();
        let expected_semantic_errs = vec![
            SemanticError::ModelIdFieldMissing {
                span: Span::new(45, 298),
                model_name: "User",
            },
            SemanticError::AttributeIncompatible {
                span: Span::new(102, 117),
                attrib_name: "unknown_attrib",
                first_attrib_name: "unique",
                field_name: "email",
                model_name: "User",
            },
            SemanticError::AttributeArgInvalid {
                span: Span::new(196, 200),
                attrib_arg_name: Some("USER"),
                attrib_name: "default",
                field_name: "nickNames",
                model_name: "User",
            },
            SemanticError::EnumValueUndefined {
                span: Span::new(241, 245),
                enum_value: "Role",
                attrib_name: "default",
                field_name: "role",
                model_name: "User",
            },
            SemanticError::RelationNoAttribute {
                span: Span::new(251, 263),
                field_name: "profile",
                model_name: "User",
            },
            SemanticError::RelationNoAttribute {
                span: Span::new(276, 288),
                field_name: "posts",
                model_name: "User",
            },
            SemanticError::ModelIdFieldMissing {
                span: Span::new(361, 604),
                model_name: "Post",
            },
            SemanticError::AttributeArgInvalid {
                span: Span::new(414, 432),
                attrib_arg_name: Some("unknown_function"),
                attrib_name: "default",
                field_name: "createdAt",
                model_name: "Post",
            },
            SemanticError::TypeUndefined {
                span: Span::new(500, 504),
                type_name: "Bool",
                field_name: "published",
                model_name: "Post",
            },
            SemanticError::AttributeArgInvalid {
                span: Span::new(545, 550),
                attrib_arg_name: Some("False"),
                attrib_name: "default",
                field_name: "deleted",
                model_name: "Post",
            },
            SemanticError::RelationNoAttribute {
                span: Span::new(557, 569),
                field_name: "author",
                model_name: "Post",
            },
            SemanticError::RelationNoAttribute {
                span: Span::new(578, 590),
                field_name: "category",
                model_name: "Post",
            },
            SemanticError::ModelIdFieldMissing {
                span: Span::new(298, 361),
                model_name: "Profile",
            },
            SemanticError::RelationNoAttribute {
                span: Span::new(342, 353),
                field_name: "user",
                model_name: "Profile",
            },
            SemanticError::ModelIdFieldMissing {
                span: Span::new(604, 672),
                model_name: "Category",
            },
            SemanticError::RelationNoAttribute {
                span: Span::new(650, 662),
                field_name: "posts",
                model_name: "Category",
            },
        ];

        let decls = crate::parser::delcarations()
            .parse(&semantic_errs_sdml)
            .into_result()
            .unwrap();
        let mut ast = to_data_model(decls, true).unwrap();
        match semantic_update(&mut ast) {
            Ok(_) => assert!(false, "Expecting attribute errors to surface"),
            Err(errs) => errs
                .into_iter()
                .for_each(|e| assert!(expected_semantic_errs.contains(&e))),
        }
    }
}
