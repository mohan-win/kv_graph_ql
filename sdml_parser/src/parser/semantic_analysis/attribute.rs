use std::collections::HashMap;
use std::fmt;

use chumsky::container::Seq;

use crate::types::{
  AttribArg, Attribute, EnumDecl, FieldDecl, ModelDecl, NamedArg, Token, Type,
};

use super::err::Error;

// Valid attribute names.
pub const ATTRIB_NAME_DEFAULT: &str = "default";
pub const ATTRIB_NAME_ID: &str = "id";
pub const ATTRIB_NAME_RELATION: &str = "relation";
pub const ATTRIB_NAME_UNIQUE: &str = "unique";
pub const ATTRIB_NAME_INDEXED: &str = "indexed";
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

pub(crate) struct RelationAttributeDetails<'b> {
  pub relation_name: &'b Token,
  pub relation_scalar_field: Option<&'b FieldDecl>,
  pub referenced_model_field: Option<&'b FieldDecl>,
  pub referenced_model_relation_field: Option<&'b FieldDecl>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum AllowedFieldType {
  /// Attribute is allowed only on short (i.e. shouldn't be an array) string field.
  ScalarShortStrField { can_be_optional: bool },
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
      AllowedFieldType::ScalarShortStrField { can_be_optional } => write!(
        f,
        "{} Scalar Short String field is allowed",
        optionality_prefix(*can_be_optional)
      ),
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
pub(crate) struct AttributeDetails {
  /// Name of the attribute
  #[allow(dead_code)]
  pub name: &'static str,
  /// Compatible attributes which can be applied along with this attribute for the same field.
  pub compatible_attribute_names: Vec<&'static str>,
  /// allowed argument functions for this attribute.
  pub allowed_arg_fns: Vec<&'static str>,
  /// allowed argument values for this attribute.
  pub allowed_arg_values: Vec<&'static str>,
  /// Allowed named args for this attribute.
  pub allowed_named_args: Vec<&'static str>,
  /// Can this attribute present on a non-scalar attribute.
  pub allowed_field_type: AllowedFieldType,
}

impl AttributeDetails {
  pub fn attributes_detail_map() -> HashMap<&'static str, AttributeDetails> {
    let mut attributes_map = HashMap::new();
    attributes_map.insert(ATTRIB_NAME_DEFAULT, AttributeDetails::default_attribute());
    attributes_map.insert(ATTRIB_NAME_ID, AttributeDetails::id_attribute());
    attributes_map.insert(ATTRIB_NAME_RELATION, AttributeDetails::relation_attribute());
    attributes_map.insert(ATTRIB_NAME_UNIQUE, AttributeDetails::unique_attribute());
    attributes_map.insert(ATTRIB_NAME_INDEXED, AttributeDetails::indexed_attribute());
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
      compatible_attribute_names: vec![ATTRIB_NAME_ID, ATTRIB_NAME_INDEXED],
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
      // Only allow ShortStr as the type for ID fields. So that
      // it can directly map to GraphQL type ID field.
      allowed_field_type: AllowedFieldType::ScalarShortStrField {
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
  #[inline]
  fn indexed_attribute() -> Self {
    Self {
      name: ATTRIB_NAME_INDEXED,
      compatible_attribute_names: vec![ATTRIB_NAME_DEFAULT],
      allowed_arg_fns: vec![],
      allowed_arg_values: vec![],
      allowed_named_args: vec![],
      allowed_field_type: AllowedFieldType::AnyField {
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
