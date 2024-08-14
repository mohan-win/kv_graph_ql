use std::collections::HashMap;

use crate::ast::{FieldDecl, ModelDecl};

use super::err::SemanticError;

// Valid attribute names.
pub const ATTRIB_NAME_DEFAULT: &str = "default";
pub const ATTRIB_NAME_ID: &str = "id";
pub const ATTRIB_NAME_RELATION: &str = "relation";
pub const ATTRIB_NAME_UNIQUE: &str = "unique";
pub const ALL_VALID_ATTRIB_NAMES: [&str; 4] = [
    ATTRIB_NAME_DEFAULT,
    ATTRIB_NAME_ID,
    ATTRIB_NAME_RELATION,
    ATTRIB_NAME_UNIQUE,
];
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
    unimplemented!()
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
    pub fn attribute_detail_map() -> HashMap<&'static str, AttributeDetails> {
        let mut attributes_map = HashMap::new();
        attributes_map.insert(ATTRIB_NAME_DEFAULT, AttributeDetails::default_attribute());
        attributes_map.insert(ATTRIB_NAME_ID, AttributeDetails::id_attribute());
        attributes_map.insert(ATTRIB_NAME_RELATION, AttributeDetails::relation_attribute());
        attributes_map.insert(ATTRIB_NAME_UNIQUE, AttributeDetails::unique_attribute());
        attributes_map
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
