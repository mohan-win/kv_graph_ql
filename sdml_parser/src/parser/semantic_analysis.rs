use crate::ast::{Attribute, DataModel, Declaration, EnumDecl, FieldDecl, ModelDecl, Token, Type};
use core::fmt;
use std::collections::{HashMap, HashSet};

/// Type of the semantic error
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticError<'src> {
    /// This error is returned when name of a user defined type clashes with already existing type.
    DuplicateTypeDefinition { type_name: &'src str },
    /// This error is returned if type of a field is undefined.
    UndefinedType {
        type_name: &'src str,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// This error is returned if undefined enum value is used.
    UndefinedEnumValue {
        enum_value: &'src str,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// This error is thrown if the attribute is invalid
    InvalidAttribute {
        attrib_name: &'src str,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// This error is returned for unknown attribute usage in models.
    UnknownAttribute {
        attrib_name: &'src str,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// This error is returned for unknown functions usage in the model attributes.
    UnknownFunction {
        fn_name: &'src str,
        attrib_name: &'src str,
        field_name: &'src str,
        model_name: &'src str,
    },
}

impl<'src> fmt::Display for SemanticError<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Semantic Error: {self:?}")
    }
}

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
    let mut type_set: HashSet<&'src str> = HashSet::new();

    let mut data_model = DataModel {
        configs: HashMap::new(),
        enums: HashMap::new(),
        models: HashMap::new(),
    };

    for decl in delcarations.into_iter() {
        let type_name = match decl {
            Declaration::Config(c) => {
                let type_name = c.name.ident_name().unwrap();
                data_model.configs.insert(type_name, c);
                type_name
            }
            Declaration::Enum(e) => {
                let type_name = e.name.ident_name().unwrap();
                data_model.enums.insert(e.name.ident_name().unwrap(), e);
                type_name
            }
            Declaration::Model(m) => {
                let type_name = m.name.ident_name().unwrap();
                data_model.models.insert(m.name.ident_name().unwrap(), m);
                type_name
            }
        };

        // error for duplicate types.
        if check_duplicate_types {
            if type_set.contains(type_name) {
                errs.push(SemanticError::DuplicateTypeDefinition { type_name });
            } else {
                type_set.insert(type_name);
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
    input_ast: &DataModel<'src>,
) -> Result<(), Vec<SemanticError<'src>>> {
    let mut errs: Vec<SemanticError<'src>> = Vec::new();
    input_ast.models.values().for_each(|model| {
        model.fields.iter().for_each(|field| {
            get_actual_type(field, &model.name, &input_ast.models, &input_ast.enums).map_or_else(
                |err| errs.push(err),
                |actual_type| {
                    actual_type.map(|actual_type| field.field_type.set_type(actual_type));
                },
            );
            field.attributes.iter().for_each(|attribute| {
                let _ = validate_attribute(attribute, &field.name, &model.name, &input_ast.enums)
                    .map_err(|attrib_err| {
                        errs.push(attrib_err);
                    });
            });
        });
    });

    if errs.len() > 1 {
        Err(errs)
    } else {
        Ok(())
    }
}

/// This function finds and returns the actual type for the field if it's type is Unknown
/// If the field type is already known, it returns `None`.
/// But if its unable to locate the Uknown type, then it returns SemanticError::UndefinedType.
fn get_actual_type<'src>(
    field: &FieldDecl<'src>,
    parent_model_ident: &Token<'src>,
    models: &HashMap<&'src str, ModelDecl<'src>>,
    enums: &HashMap<&'src str, EnumDecl<'src>>,
) -> Result<Option<Type<'src>>, SemanticError<'src>> {
    if let Type::Unknown(type_name_tok) = &*field.field_type.r#type() {
        let type_name = type_name_tok.ident_name().unwrap();
        match models.get(type_name) {
            Some(_) => Ok(Some(Type::Relation(type_name_tok.clone()))),
            None => match enums.get(type_name) {
                Some(_) => Ok(Some(Type::Enum(type_name_tok.clone()))),
                None => Err(SemanticError::UndefinedType {
                    type_name: type_name_tok.ident_name().unwrap(),
                    field_name: field.name.ident_name().unwrap(),
                    model_name: parent_model_ident.ident_name().unwrap(),
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
    parent_field_ident: &Token<'src>,
    parent_model_ident: &Token<'src>,
    enums: &HashMap<&'src str, EnumDecl<'src>>,
) -> Result<(), SemanticError<'src>> {
    let valid_attribs_with_no_args = ["unique"];
    let valid_attributes_with_args = ["default"];
    let valid_attribute_arg_fns = ["now"];
    let mut all_attributes = Vec::new();
    all_attributes.extend_from_slice(&valid_attribs_with_no_args[..]);
    all_attributes.extend_from_slice(&valid_attributes_with_args[..]);

    let attribute_name = attribute.name.ident_name().unwrap();

    // See if it is a known attribute
    if !all_attributes.contains(&attribute_name) {
        Err(SemanticError::UnknownAttribute {
            attrib_name: attribute_name,
            field_name: parent_field_ident.ident_name().unwrap(),
            model_name: parent_model_ident.ident_name().unwrap(),
        })
    } else {
        // See if it is an invalid attribute
        if valid_attribs_with_no_args.contains(&attribute_name) && attribute.arg.is_some()
            || valid_attributes_with_args.contains(&attribute_name) && attribute.arg.is_none()
        {
            Err(SemanticError::InvalidAttribute {
                attrib_name: attribute.name.ident_name().unwrap(),
                field_name: parent_field_ident.ident_name().unwrap(),
                model_name: parent_model_ident.ident_name().unwrap(),
            })
        } else if valid_attributes_with_args.contains(&attribute_name) && attribute.arg.is_some() {
            // See if it has valid args
            let attribute_arg_name = attribute.arg.as_ref().unwrap().name.ident_name().unwrap();
            if !attribute.arg.as_ref().unwrap().is_function
                && !enums.contains_key(attribute_arg_name)
            {
                Err(SemanticError::UndefinedEnumValue {
                    enum_value: attribute_arg_name,
                    field_name: parent_field_ident.ident_name().unwrap(),
                    model_name: parent_model_ident.ident_name().unwrap(),
                })
            } else if attribute.arg.as_ref().unwrap().is_function
                && !valid_attribute_arg_fns.contains(&attribute_arg_name)
            {
                Err(SemanticError::UnknownFunction {
                    fn_name: attribute_arg_name,
                    attrib_name: attribute_name,
                    field_name: parent_field_ident.ident_name().unwrap(),
                    model_name: parent_model_ident.ident_name().unwrap(),
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chumsky::prelude::*;

    #[test]
    fn test_duplicate_types() {
        let duplicate_types_sdml = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/sdml/semantic_analysis/duplicate_types.sdml"
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
                        SemanticError::DuplicateTypeDefinition { type_name: "db" },
                        SemanticError::DuplicateTypeDefinition { type_name: "User" },
                        SemanticError::DuplicateTypeDefinition { type_name: "Role" }
                    ]
                )
            }
        }
    }

    #[test]
    fn test_semantic_update() {
        let semantic_errs_sdml = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/sdml/semantic_analysis/semantic_errs.sdml"
        ))
        .unwrap();

        let decls = crate::parser::delcarations()
            .parse(&semantic_errs_sdml)
            .into_result()
            .unwrap();
        let ast = to_data_model(decls, true).unwrap();
        match semantic_update(&ast) {
            Ok(_) => assert!(false, "Expecting attribute errors to surface"),
            Err(errs) => {
                let expected_errs = vec![
                    SemanticError::InvalidAttribute {
                        attrib_name: "unique",
                        field_name: "email",
                        model_name: "User",
                    },
                    SemanticError::UnknownAttribute {
                        attrib_name: "unknown_attrib",
                        field_name: "email",
                        model_name: "User",
                    },
                    SemanticError::InvalidAttribute {
                        attrib_name: "default",
                        field_name: "name",
                        model_name: "User",
                    },
                    SemanticError::UndefinedEnumValue {
                        enum_value: "UNDEFINED_ENUM",
                        field_name: "role",
                        model_name: "User",
                    },
                    SemanticError::UnknownFunction {
                        fn_name: "unknown_function",
                        field_name: "createdAt",
                        attrib_name: "default",
                        model_name: "Post",
                    },
                    SemanticError::UndefinedType {
                        type_name: "Bool",
                        field_name: "published",
                        model_name: "Post",
                    },
                ];
                assert_eq!(errs.len(), expected_errs.len());
                errs.iter().for_each(|e| assert!(expected_errs.contains(e)))
            }
        }
    }
}
