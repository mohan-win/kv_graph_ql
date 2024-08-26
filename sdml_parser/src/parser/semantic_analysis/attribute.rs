use std::collections::HashMap;

use crate::ast::{AttribArg, Attribute, FieldDecl, ModelDecl};

use super::err::SemanticError;

// Valid attribute names.
pub const ATTRIB_NAME_DEFAULT: &str = "default";
pub const ATTRIB_NAME_ID: &str = "id";
pub const ATTRIB_NAME_RELATION: &str = "relation";
pub const ATTRIB_NAME_UNIQUE: &str = "unique";
// Valid attribute arg functions
pub const ATTRIB_ARG_FN_NOW: &str = "now";
pub const ATTRIB_ARG_FN_AUTO: &str = "auto";

// Valid attribute arg values
pub const ATTRIB_ARG_VALUE_TRUE: &str = "true";
pub const ATTRIB_ARG_VALUE_FALSE: &str = "false";
/// Allow valid enum value if attribute is present on field whos type is an enum.
pub const ATTRIB_ARG_VALUE_ENUM: &str = "enum";

// Valid attribute named args
pub const ATTRIB_NAMED_ARG_NAME: &str = "name";
pub const ATTRIB_NAMED_ARG_FIELD: &str = "field";
pub const ATTRIB_NAMED_ARG_REFERENCES: &str = "references";

/// Validates the attributes of the given field in the model.
pub fn validate_attributes<'src>(
    field: &FieldDecl<'src>,
    model: &ModelDecl<'src>,
) -> Result<(), SemanticError<'src>> {
    #[allow(non_snake_case)]
    let ATTRIBUTES_DETAIL_MAP = AttributeDetails::attributes_detail_map();
    is_attributes_compatible(field, model, &ATTRIBUTES_DETAIL_MAP)?;
    field.attributes.iter().try_for_each(|attribute| {
        validate_attribute_args(attribute, field, model, &ATTRIBUTES_DETAIL_MAP)
    })
}

/// Returns if the attributes present in the array are compatible with each other,
/// so that they (all of them present in array) can be applied on a single field.
fn is_attributes_compatible<'src>(
    field: &FieldDecl<'src>,
    model: &ModelDecl<'src>,
    attribute_details_map: &HashMap<&'static str, AttributeDetails>,
) -> Result<(), SemanticError<'src>> {
    if field.attributes.len() == 0 {
        Ok(())
    } else {
        let first_attribute = &field.attributes[0];
        match attribute_details_map.get(first_attribute.name.ident_name().unwrap()) {
            Some(attribute_detail) => {
                if field.attributes.len() > 1 {
                    field.attributes[1..].iter().try_for_each(|attribute| {
                        if !attribute_detail
                            .compatible_attribute_names
                            .contains(&attribute.name.ident_name().unwrap())
                        {
                            Err(SemanticError::AttributeIncompatible {
                                span: attribute.name.span(),
                                attrib_name: attribute.name.ident_name().unwrap(),
                                first_attrib_name: first_attribute.name.ident_name().unwrap(),
                                field_name: field.name.ident_name().unwrap(),
                                model_name: model.name.ident_name().unwrap(),
                            })
                        } else {
                            Ok(())
                        }
                    })
                } else {
                    Ok(())
                }
            }
            None => Err(SemanticError::AttributeUnknown {
                span: first_attribute.name.span(),
                attrib_name: first_attribute.name.ident_name().unwrap(),
                field_name: field.name.ident_name().unwrap(),
                model_name: model.name.ident_name().unwrap(),
            }),
        }
    }
}

/// Validate the attribute arguments if any.
fn validate_attribute_args<'src>(
    attrib: &Attribute<'src>,
    field: &FieldDecl<'src>,
    model: &ModelDecl<'src>,
    attribute_details_map: &HashMap<&'static str, AttributeDetails>,
) -> Result<(), SemanticError<'src>> {
    match attribute_details_map.get(attrib.name.ident_name().unwrap()) {
        None => Err(SemanticError::AttributeUnknown {
            span: attrib.name.span(),
            attrib_name: attrib.name.ident_name().unwrap(),
            field_name: field.name.ident_name().unwrap(),
            model_name: model.name.ident_name().unwrap(),
        }),
        Some(attrib_detail) => {
            if attrib
                .arg
                .as_ref()
                .is_some_and(|_| attrib_detail.is_empty_arg_attribute())
            {
                // Is this an empty arg attribute incorrectly having some arguments ?
                Err(SemanticError::AttributeArgInvalid {
                    span: attrib.name.span(),
                    attrib_arg_name: None,
                    attrib_name: attrib.name.ident_name().unwrap(),
                    field_name: field.name.ident_name().unwrap(),
                    model_name: model.name.ident_name().unwrap(),
                })
            } else if let Some(attrib_arg) = attrib.arg.as_ref() {
                match attrib_arg {
                    AttribArg::Function(fn_name) => {
                        if !attrib_detail
                            .allowed_arg_fns
                            .contains(&fn_name.ident_name().unwrap())
                        {
                            Err(SemanticError::AttributeArgInvalid {
                                span: fn_name.span(),
                                attrib_arg_name: Some(fn_name.ident_name().unwrap()),
                                attrib_name: attrib.name.ident_name().unwrap(),
                                field_name: field.name.ident_name().unwrap(),
                                model_name: model.name.ident_name().unwrap(),
                            })
                        } else {
                            Ok(())
                        }
                    }
                    AttribArg::Ident(arg_value) => {
                        if !attrib_detail
                            .allowed_arg_values
                            .contains(&arg_value.ident_name().unwrap())
                        {
                            Err(SemanticError::AttributeArgInvalid {
                                span: arg_value.span(),
                                attrib_arg_name: Some(arg_value.ident_name().unwrap()),
                                attrib_name: attrib.name.ident_name().unwrap(),
                                field_name: field.name.ident_name().unwrap(),
                                model_name: model.name.ident_name().unwrap(),
                            })
                        } else {
                            Ok(())
                        }
                    }
                    AttribArg::Args(named_args) => {
                        let mut invalid_args = named_args.iter().filter_map(|named_arg| {
                            if !attrib_detail
                                .allowed_named_args
                                .contains(&named_arg.arg_name.ident_name().unwrap())
                            {
                                Some(&named_arg.arg_name)
                            } else {
                                None
                            }
                        });
                        if let Some(invalid_arg) = invalid_args.next() {
                            Err(SemanticError::AttributeArgInvalid {
                                span: invalid_arg.span(),
                                attrib_arg_name: Some(invalid_arg.ident_name().unwrap()),
                                attrib_name: attrib.name.ident_name().unwrap(),
                                field_name: field.name.ident_name().unwrap(),
                                model_name: model.name.ident_name().unwrap(),
                            })
                        } else {
                            Ok(())
                        }
                    }
                }
            } else {
                Ok(())
            }
        }
    }
}

/// Type to capture details of an attribute.
#[derive(Debug)]
struct AttributeDetails {
    /// Name of the attribute
    name: &'static str,
    /// Compatible attributes which can be applied along with this attribute for the same field.
    compatible_attribute_names: Vec<&'static str>,
    /// allowed argument functions for this attribute.
    allowed_arg_fns: Vec<&'static str>,
    /// allowed argument values for this attribute.
    allowed_arg_values: Vec<&'static str>,
    /// Allowed named args for this attribute.
    allowed_named_args: Vec<&'static str>,
}

impl AttributeDetails {
    pub fn attributes_detail_map() -> HashMap<&'static str, AttributeDetails> {
        let mut attributes_map = HashMap::new();
        attributes_map.insert(ATTRIB_NAME_DEFAULT, AttributeDetails::default_attribute());
        attributes_map.insert(ATTRIB_NAME_ID, AttributeDetails::id_attribute());
        attributes_map.insert(ATTRIB_NAME_RELATION, AttributeDetails::relation_attribute());
        attributes_map.insert(ATTRIB_NAME_UNIQUE, AttributeDetails::unique_attribute());
        attributes_map
    }
    /// Does this attribute shouldn't have any args ?
    pub fn is_empty_arg_attribute(&self) -> bool {
        self.allowed_arg_fns.len() == 0
            && self.allowed_arg_values.len() == 0
            && self.allowed_named_args.len() == 0
    }
    #[inline]
    fn default_attribute() -> Self {
        Self {
            name: ATTRIB_NAME_DEFAULT,
            compatible_attribute_names: vec![ATTRIB_NAME_ID],
            allowed_arg_fns: vec![ATTRIB_ARG_FN_AUTO, ATTRIB_ARG_FN_NOW],
            allowed_arg_values: vec![
                ATTRIB_ARG_VALUE_TRUE,
                ATTRIB_ARG_VALUE_FALSE,
                ATTRIB_ARG_VALUE_ENUM,
            ],
            allowed_named_args: vec![],
        }
    }
    #[inline]
    fn id_attribute() -> Self {
        Self {
            name: ATTRIB_NAME_ID,
            compatible_attribute_names: vec![ATTRIB_NAME_DEFAULT],
            allowed_arg_fns: vec![],
            allowed_arg_values: vec![],
            allowed_named_args: vec![],
        }
    }
    #[inline]
    fn relation_attribute() -> Self {
        Self {
            name: ATTRIB_NAME_RELATION,
            compatible_attribute_names: vec![],
            allowed_arg_fns: vec![],
            allowed_arg_values: vec![],
            allowed_named_args: vec![
                ATTRIB_NAMED_ARG_NAME,
                ATTRIB_NAMED_ARG_FIELD,
                ATTRIB_NAMED_ARG_REFERENCES,
            ],
        }
    }
    #[inline]
    fn unique_attribute() -> Self {
        Self {
            name: ATTRIB_NAME_UNIQUE,
            compatible_attribute_names: vec![],
            allowed_arg_fns: vec![],
            allowed_arg_values: vec![],
            allowed_named_args: vec![],
        }
    }
}
