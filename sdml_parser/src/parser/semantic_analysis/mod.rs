use crate::ast::{
    Attribute, DataModel, Declaration, EnumDecl, FieldDecl, ModelDecl, RelationEdge, Span, Token,
    Type,
};
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    ops::{ControlFlow, Deref},
};

/// Error Module
pub mod err;
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
                errs.push(SemanticError::DuplicateTypeDefinition { span, type_name });
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
            let _ = attribute::validate_attributes(field, model).map_err(|err| errs.push(err));
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
                Some(_) => Ok(Some(Type::Enum(type_name_tok.clone()))), // Clone
                None => Err(SemanticError::UndefinedType {
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

/// Validates the given attribute on the model fields, returns error if
/// a. it finds a unknown attribute
/// b. if finds a unknown function
/// c. it finds a unknown Enum value.
fn validate_attribute<'src>(
    attribute: &Attribute<'src>,
    parent_field: &FieldDecl<'src>,
    parent_model_ident: &Token<'src>,
    enums: &HashMap<&'src str, EnumDecl<'src>>,
) -> Result<(), SemanticError<'src>> {
    unimplemented!()
    /*let valid_attribs_with_no_args = ["unique"];
    let valid_attributes_with_args = ["default"];
    let valid_attribute_arg_fns = ["now"];
    let valid_attribute_arg_values = ["true", "false"];
    let mut all_attributes = Vec::new();
    all_attributes.extend_from_slice(&valid_attribs_with_no_args[..]);
    all_attributes.extend_from_slice(&valid_attributes_with_args[..]);

    let attribute_name = attribute.name.ident_name().unwrap();

    // See if it is a known attribute
    if !all_attributes.contains(&attribute_name) {
        Err(SemanticError::UnknownAttribute {
            span: attribute.name.span(),
            attrib_name: attribute_name,
            field_name: parent_field.name.ident_name().unwrap(),
            model_name: parent_model_ident.ident_name().unwrap(),
        })
    } else {
        // See if it is an invalid attribute
        if valid_attribs_with_no_args.contains(&attribute_name) && attribute.arg.is_some()
            || valid_attributes_with_args.contains(&attribute_name) && attribute.arg.is_none()
        {
            Err(SemanticError::InvalidAttribute {
                span: attribute.name.span(),
                attrib_name: attribute.name.ident_name().unwrap(),
                field_name: parent_field.name.ident_name().unwrap(),
                model_name: parent_model_ident.ident_name().unwrap(),
            })
        } else if valid_attributes_with_args.contains(&attribute_name) && attribute.arg.is_some() {
            // See if the arg is a valid function.
            let attribute_arg = &attribute.arg.as_ref().unwrap().name;
            let attribute_arg_name = attribute_arg.ident_name().unwrap();
            if attribute.arg.as_ref().unwrap().is_function
                && !valid_attribute_arg_fns.contains(&attribute_arg_name)
            {
                Err(SemanticError::UnknownFunction {
                    span: attribute_arg.span(),
                    fn_name: attribute_arg_name,
                    attrib_name: attribute_name,
                    field_name: parent_field.name.ident_name().unwrap(),
                    model_name: parent_model_ident.ident_name().unwrap(),
                })
            } else if !attribute.arg.as_ref().unwrap().is_function {
                if let Type::Primitive {
                    r#type: PrimitiveType::Boolean,
                    ..
                } = &*parent_field.field_type.r#type()
                {
                    // See if the arg is a valid boolean value.
                    if !valid_attribute_arg_values.contains(&attribute_arg_name) {
                        Err(SemanticError::InvalidAttributeArg {
                            span: attribute_arg.span(),
                            attrib_arg_name: attribute_arg_name,
                            attrib_name: attribute_name,
                            field_name: parent_field.name.ident_name().unwrap(),
                            model_name: parent_model_ident.ident_name().unwrap(),
                        })
                    } else {
                        Ok(())
                    }
                } else if let Type::Enum(enum_name) = &*parent_field.field_type.r#type() {
                    // See if the arg is a valid enum value of the given type.
                    if let Some((_, enum_decl)) = enums.get_key_value(
                        enum_name
                            .ident_name()
                            .expect("Enum should have an identifier"),
                    ) {
                        if enum_decl.elements.contains(attribute_arg) {
                            Ok(())
                        } else {
                            Err(SemanticError::UndefinedEnumValue {
                                span: attribute_arg.span(),
                                enum_value: attribute_arg_name,
                                attrib_name: attribute_name,
                                field_name: parent_field.name.ident_name().unwrap(),
                                model_name: parent_model_ident.ident_name().unwrap(),
                            })
                        }
                    } else {
                        Err(SemanticError::UndefinedEnum {
                            span: attribute_arg.span(),
                            r#enum: enum_name.ident_name().unwrap(),
                            field_name: parent_field.name.ident_name().unwrap(),
                            model_name: parent_model_ident.ident_name().unwrap(),
                        })
                    }
                } else {
                    Err(SemanticError::InvalidAttributeArg {
                        span: attribute_arg.span(),
                        attrib_arg_name: attribute_arg_name,
                        attrib_name: attribute_name,
                        field_name: parent_field.name.ident_name().unwrap(),
                        model_name: parent_model_ident.ident_name().unwrap(),
                    })
                }
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }*/
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
                        SemanticError::DuplicateTypeDefinition {
                            span: Span::new(52, 54),
                            type_name: "db"
                        },
                        SemanticError::DuplicateTypeDefinition {
                            span: Span::new(294, 311),
                            type_name: "User"
                        },
                        SemanticError::DuplicateTypeDefinition {
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

        let decls = crate::parser::delcarations()
            .parse(&semantic_errs_sdml)
            .into_result()
            .unwrap();
        let mut ast = to_data_model(decls, true).unwrap();
        match semantic_update(&mut ast) {
            Ok(_) => assert!(false, "Expecting attribute errors to surface"),
            Err(errs) => {
                let expected_errs = vec![
                    SemanticError::InvalidAttribute {
                        span: Span::new(88, 101),
                        attrib_name: "unique",
                        field_name: "email",
                        model_name: "User",
                    },
                    SemanticError::UnknownAttribute {
                        span: Span::new(102, 117),
                        attrib_name: "unknown_attrib",
                        field_name: "email",
                        model_name: "User",
                    },
                    SemanticError::InvalidAttribute {
                        span: Span::new(148, 156),
                        attrib_name: "default",
                        field_name: "name",
                        model_name: "User",
                    },
                    SemanticError::InvalidAttributeArg {
                        span: Span::new(186, 200),
                        attrib_arg_name: "USER",
                        attrib_name: "default",
                        field_name: "nickNames",
                        model_name: "User",
                    },
                    SemanticError::UndefinedEnumValue {
                        span: Span::new(231, 245),
                        enum_value: "Role",
                        attrib_name: "default",
                        field_name: "role",
                        model_name: "User",
                    },
                    SemanticError::UnknownFunction {
                        span: Span::new(404, 432),
                        fn_name: "unknown_function",
                        field_name: "createdAt",
                        attrib_name: "default",
                        model_name: "Post",
                    },
                    SemanticError::UndefinedType {
                        span: Span::new(499, 503),
                        type_name: "Bool",
                        field_name: "published",
                        model_name: "Post",
                    },
                    SemanticError::InvalidAttributeArg {
                        span: Span::new(535, 550),
                        attrib_arg_name: "False",
                        attrib_name: "default",
                        field_name: "deleted",
                        model_name: "Post",
                    },
                ];

                assert_eq!(expected_errs.len(), errs.len());
                errs.iter()
                    .for_each(|err| assert!(expected_errs.contains(err)))
            }
        }
    }
}
