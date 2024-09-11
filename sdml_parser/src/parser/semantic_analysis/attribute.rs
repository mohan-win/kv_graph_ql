use std::collections::HashMap;
use std::fmt;

use crate::ast::{
    AttribArg, Attribute, EnumDecl, FieldDecl, ModelDecl, NamedArg, Span, Token, Type,
};

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

pub(crate) struct RelationAttributeDetails<'src, 'b> {
    pub relation_name: Option<Token<'src>>,
    pub relation_scalar_field: Option<&'b FieldDecl<'src>>,
    pub referenced_model_field: Option<&'b FieldDecl<'src>>,
}
/// Validates arguments passed to @relation attribute
pub(crate) fn validate_relation_attribute_args<'src, 'b>(
    relation_args: &'b Vec<NamedArg<'src>>,
    field: &'b FieldDecl<'src>,
    model: &'b ModelDecl<'src>,
    referenced_model: &'b ModelDecl<'src>,
) -> Result<RelationAttributeDetails<'src, 'b>, SemanticError<'src>> {
    let mut relation_name = None;
    let mut relation_scalar_field: Option<&'b FieldDecl<'src>> = None;
    let mut referenced_model_field: Option<&'b FieldDecl<'src>> = None;
    for arg in relation_args.iter() {
        match arg.arg_name {
            Token::Ident(ATTRIB_NAMED_ARG_NAME, _) => {
                if let Token::Str(..) = arg.arg_value {
                    relation_name = Some(arg.arg_value.clone())
                } else {
                    return Err(SemanticError::RelationInvalidAttributeArg {
                        span: arg.arg_value.span(),
                        relation_name: None,
                        field_name: Some(field.name.ident_name().unwrap()),
                        model_name: Some(model.name.ident_name().unwrap()),
                    });
                }
            }
            Token::Ident(ATTRIB_NAMED_ARG_FIELD, _) => {
                relation_scalar_field = Some(get_relation_scalar_field(arg, field, model)?);
            }
            Token::Ident(ATTRIB_NAMED_ARG_REFERENCES, _) => {
                referenced_model_field = Some(get_referenced_model_field(
                    arg,
                    field,
                    model,
                    referenced_model,
                )?);
            }
            _ => {
                return Err(SemanticError::RelationInvalidAttributeArg {
                    span: arg.arg_value.span(),
                    relation_name: None,
                    field_name: Some(field.name.ident_name().unwrap()),
                    model_name: Some(model.name.ident_name().unwrap()),
                })
            }
        }
    }

    // Make sure relation scalar field and referenced field are of the `same primitive type`
    if relation_scalar_field.is_some()
        && referenced_model_field.is_some()
        && *relation_scalar_field.unwrap().field_type.r#type()
            != *referenced_model_field.unwrap().field_type.r#type()
    {
        Err(
            SemanticError::RelationScalarAndReferencedFieldsTypeMismatch {
                span: relation_scalar_field.unwrap().name.span(),
                field_name: relation_scalar_field.unwrap().name.ident_name().unwrap(),
                model_name: model.name.ident_name().unwrap(),
                referenced_field_name: referenced_model_field.unwrap().name.ident_name().unwrap(),
                referenced_model_name: referenced_model.name.ident_name().unwrap(),
            },
        )
    } else {
        Ok(RelationAttributeDetails {
            relation_name,
            relation_scalar_field,
            referenced_model_field,
        })
    }
}

fn get_relation_scalar_field<'src, 'b>(
    relation_arg_field: &'b NamedArg<'src>,
    field: &'b FieldDecl<'src>,
    model: &'b ModelDecl<'src>,
) -> Result<&'b FieldDecl<'src>, SemanticError<'src>> {
    debug_assert!(
        Token::Ident(ATTRIB_NAMED_ARG_FIELD, Span::new(0, 0)) == relation_arg_field.arg_name,
        "Invalid argument passed for relation_arg_field"
    );

    let relation_scalar_field: Option<&'b FieldDecl<'src>>;
    // Validate relation scalar field.
    // Make sure relation scalar field exists in the parent model.
    if let Token::Ident(..) = relation_arg_field.arg_value {
        relation_scalar_field = model
            .fields
            .iter()
            .find(|field| field.name == relation_arg_field.arg_value);
        if relation_scalar_field.is_none() {
            Err(SemanticError::RelationScalarFieldNotFound {
                span: relation_arg_field.arg_value.span(),
                field_name: field.name.ident_name().unwrap(),
                model_name: model.name.ident_name().unwrap(),
            })
        } else if relation_scalar_field.is_some_and(|relation_scalar_field| {
            // If relation_scalar_field is not a scalar, then throw error.
            match &*relation_scalar_field.field_type.r#type() {
                Type::Primitive { .. } => false,
                _ => true,
            }
        }) {
            Err(SemanticError::RelationScalarFieldIsNotScalar {
                span: relation_scalar_field.unwrap().name.span(),
                field_name: relation_scalar_field.unwrap().name.ident_name().unwrap(),
                model_name: model.name.ident_name().unwrap(),
            })
        } else if relation_scalar_field.is_some_and(|relation_scalar_field| {
            relation_scalar_field.field_type.is_array
                && relation_scalar_field
                    .attributes
                    .iter()
                    .find(|attrib| match attrib.name {
                        Token::Ident(ATTRIB_NAME_UNIQUE, ..) => true,
                        _ => false,
                    })
                    .is_some()
        }) {
            Err(SemanticError::RelationScalarFieldArrayCanNotBeUnique {
                span: relation_scalar_field.unwrap().name.span(),
                field_name: relation_scalar_field.unwrap().name.ident_name().unwrap(),
                model_name: model.name.ident_name().unwrap(),
            })
        } else {
            Ok(relation_scalar_field.unwrap())
        }
    } else {
        Err(SemanticError::RelationInvalidAttributeArg {
            span: relation_arg_field.arg_value.span(),
            relation_name: None,
            field_name: Some(field.name.ident_name().unwrap()),
            model_name: Some(model.name.ident_name().unwrap()),
        })
    }
}

fn get_referenced_model_field<'src, 'b>(
    relation_arg_references: &'b NamedArg<'src>,
    field: &'b FieldDecl<'src>,
    model: &'b ModelDecl<'src>,
    referenced_model: &'b ModelDecl<'src>,
) -> Result<&'b FieldDecl<'src>, SemanticError<'src>> {
    debug_assert!(
        Token::Ident(ATTRIB_NAMED_ARG_REFERENCES, Span::new(0, 0))
            == relation_arg_references.arg_name,
        "Invalid arg is passed for relation_arg_references"
    );
    let referenced_model_field: Option<&'b FieldDecl<'src>>;
    // Validate referenced field.
    // Make sure referenced field, exists in the referenced model.
    if let Token::Ident(_, _) = relation_arg_references.arg_value {
        referenced_model_field = referenced_model
            .fields
            .iter()
            .find(|field| relation_arg_references.arg_value == field.name);
        if referenced_model_field.is_none() {
            Err(SemanticError::RelationReferencedFieldNotFound {
                span: relation_arg_references.arg_value.span(),
                field_name: field.name.ident_name().unwrap(),
                model_name: model.name.ident_name().unwrap(),
                referenced_field_name: relation_arg_references.arg_value.ident_name().unwrap(),
                referenced_model_name: referenced_model.name.ident_name().unwrap(),
            })
        } else if referenced_model_field.is_some_and(|referenced_model_field| {
            match &*referenced_model_field.field_type.r#type() {
                Type::Primitive { .. } => false,
                _ => true,
            }
        }) {
            // If referenced field is not scalar throw an error
            Err(SemanticError::RelationReferencedFieldNotScalar {
                span: relation_arg_references.arg_value.span(),
                field_name: field.name.ident_name().unwrap(),
                model_name: model.name.ident_name().unwrap(),
                referenced_field_name: referenced_model_field.unwrap().name.ident_name().unwrap(),
                referenced_model_name: referenced_model.name.ident_name().unwrap(),
            })
        } else if referenced_model_field.is_some_and(|referenced_model_field| {
            referenced_model_field
                .attributes
                .iter()
                .find(|attrib| match attrib.name {
                    Token::Ident(ATTRIB_NAME_UNIQUE, ..) | Token::Ident(ATTRIB_NAME_ID, ..) => true,
                    _ => false,
                })
                .is_none()
        }) {
            // if the referenced field is not attributed with @id or @unique then throw error.
            Err(SemanticError::RelationReferencedFieldNotUnique {
                span: relation_arg_references.arg_value.span(),
                field_name: field.name.ident_name().unwrap(),
                model_name: model.name.ident_name().unwrap(),
                referenced_field_name: referenced_model_field.unwrap().name.ident_name().unwrap(),
                referenced_model_name: referenced_model.name.ident_name().unwrap(),
            })
        } else {
            Ok(referenced_model_field.unwrap())
        }
    } else {
        Err(SemanticError::RelationInvalidAttributeArg {
            span: relation_arg_references.arg_value.span(),
            relation_name: None,
            field_name: Some(field.name.ident_name().unwrap()),
            model_name: Some(model.name.ident_name().unwrap()),
        })
    }
}

/// Validates the attributes of the given field in the model.
pub(crate) fn validate_attributes<'src>(
    field: &FieldDecl<'src>,
    model: &ModelDecl<'src>,
    enums: &HashMap<&'src str, EnumDecl<'src>>,
) -> Result<(), SemanticError<'src>> {
    #[allow(non_snake_case)]
    let ATTRIBUTES_DETAIL_MAP = AttributeDetails::attributes_detail_map();
    is_attributes_compatible(field, model, &ATTRIBUTES_DETAIL_MAP)?;
    field.attributes.iter().try_for_each(|attribute| {
        is_valid_field_type(attribute, field, model, &ATTRIBUTES_DETAIL_MAP).and_then(|()| {
            validate_attribute_args(attribute, field, model, enums, &ATTRIBUTES_DETAIL_MAP)
        })
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

/// Is the field_type valid for the given attribute ?
fn is_valid_field_type<'src>(
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
            let is_scalar_field = field.field_type.r#type().is_scalar_type();
            let is_optional_field = field.field_type.is_optional;
            match attrib_detail.allowed_field_type {
                AllowedFieldType::ScalarField { can_be_optional }
                    if is_scalar_field && (can_be_optional == is_optional_field) =>
                {
                    Ok(())
                }
                AllowedFieldType::NonScalarField { can_be_optional }
                    if !is_scalar_field && (can_be_optional == is_optional_field) =>
                {
                    Ok(())
                }
                _ => Err(SemanticError::AttributeInvalid {
                    span: attrib.name.span(),
                    reason: attrib_detail.allowed_field_type.to_string(),
                    attrib_name: attrib.name.ident_name().unwrap(),
                    field_name: field.name.ident_name().unwrap(),
                    model_name: model.name.ident_name().unwrap(),
                }),
            }
        }
    }
}

/// Validate the attribute arguments if any.
fn validate_attribute_args<'src>(
    attrib: &Attribute<'src>,
    field: &FieldDecl<'src>,
    model: &ModelDecl<'src>,
    enums: &HashMap<&'src str, EnumDecl<'src>>,
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
                        if let Type::Enum { enum_ty_name } = &*field.field_type.r#type() {
                            // Is enum values are allowed as arg values ?
                            if !attrib_detail
                                .allowed_arg_values
                                .contains(&ATTRIB_ARG_VALUE_ENUM)
                            {
                                Err(SemanticError::AttributeArgInvalid {
                                    span: arg_value.span(),
                                    attrib_arg_name: Some(arg_value.ident_name().unwrap()),
                                    attrib_name: attrib.name.ident_name().unwrap(),
                                    field_name: field.name.ident_name().unwrap(),
                                    model_name: model.name.ident_name().unwrap(),
                                })
                            } else {
                                let enum_decl = enums
                                    .get(enum_ty_name.ident_name().unwrap())
                                    .ok_or_else(|| SemanticError::EnumUndefined {
                                        span: enum_ty_name.span(),
                                        r#enum: enum_ty_name.ident_name().unwrap(),
                                        field_name: field.name.ident_name().unwrap(),
                                        model_name: model.name.ident_name().unwrap(),
                                    })?;
                                if enum_decl.elements.contains(arg_value) {
                                    Ok(())
                                } else {
                                    Err(SemanticError::EnumValueUndefined {
                                        span: arg_value.span(),
                                        enum_value: arg_value.ident_name().unwrap(),
                                        attrib_name: attrib.name.ident_name().unwrap(),
                                        field_name: field.name.ident_name().unwrap(),
                                        model_name: model.name.ident_name().unwrap(),
                                    })
                                }
                            }
                        } else if !attrib_detail
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

#[derive(Debug, PartialEq)]
enum AllowedFieldType {
    /// Attribute is allowed on only scalar field.
    ScalarField { can_be_optional: bool },
    /// Attribute is allowed only on non-scalar field.
    NonScalarField { can_be_optional: bool },
    /// Attribute is allowed on both scalar and non-scalar field
    AnyField { can_be_optional: bool },
}

impl fmt::Display for AllowedFieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let optionality_prefix = |can_be_optional: bool| {
            if can_be_optional {
                "Only Optional"
            } else {
                "Only Non-Optional"
            }
        };
        // ToDo:: internationalization.
        match self {
            AllowedFieldType::ScalarField { can_be_optional } => write!(
                f,
                "{} Scalar field is allowed",
                optionality_prefix(*can_be_optional)
            ),
            AllowedFieldType::NonScalarField { can_be_optional } => {
                write!(
                    f,
                    "{} Non-Scalar field is allowed",
                    optionality_prefix(*can_be_optional)
                )
            }
            AllowedFieldType::AnyField { can_be_optional } => {
                write!(
                    f,
                    "{} field is allowed",
                    optionality_prefix(*can_be_optional)
                )
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
    /// Can this attribute present on a non-scalar attribute.
    allowed_field_type: AllowedFieldType,
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
            allowed_field_type: AllowedFieldType::ScalarField {
                can_be_optional: false,
            },
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
            allowed_field_type: AllowedFieldType::ScalarField {
                can_be_optional: false,
            },
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
            allowed_field_type: AllowedFieldType::NonScalarField {
                can_be_optional: true,
            },
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
            allowed_field_type: AllowedFieldType::ScalarField {
                can_be_optional: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_relations() {}

    #[test]
    fn test_validate_attributes() {}
}
