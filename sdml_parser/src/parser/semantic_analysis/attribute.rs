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
/// Validates arguments passed to @relation attribute
pub(crate) fn validate_relation_attribute_args<'b>(
  relation_args: &'b Vec<NamedArg>,
  field: &'b FieldDecl,
  model: &'b ModelDecl,
  referenced_model: &'b ModelDecl,
) -> Result<RelationAttributeDetails<'b>, Error> {
  let mut relation_name: Option<&'b Token> = None;
  let mut relation_scalar_field: Option<&'b FieldDecl> = None;
  let mut referenced_model_field: Option<&'b FieldDecl> = None;
  let referenced_model_relation_field: Option<&'b FieldDecl>;

  // Step 1: Validate relation attribute has correct set of args.
  let mut valid_arg_sets: HashMap<usize, Vec<_>> = HashMap::new();
  valid_arg_sets.insert(1, vec![ATTRIB_NAMED_ARG_NAME]);
  valid_arg_sets.insert(
    3,
    vec![
      ATTRIB_NAMED_ARG_NAME,
      ATTRIB_NAMED_ARG_FIELD,
      ATTRIB_NAMED_ARG_REFERENCES,
    ],
  );
  // Check for invalid arg sets
  let allowed_arg_set = valid_arg_sets.get(&relation_args.len());
  if allowed_arg_set.is_none() {
    return Err(Error::RelationInvalidAttributeArg {
      span: field.name.span(),
      relation_name: None,
      arg_name: None,
      field_name: field.name.ident_name(),
      model_name: model.name.ident_name(),
    });
  } else {
    relation_args.iter().try_for_each(|arg| {
      if allowed_arg_set
        .unwrap()
        .contains(&arg.arg_name.ident_name().unwrap().as_str())
      {
        Ok(())
      } else {
        Err(Error::RelationInvalidAttributeArg {
          span: field.name.span(),
          relation_name: None,
          arg_name: arg.arg_name.ident_name(),
          field_name: field.name.ident_name(),
          model_name: model.name.ident_name(),
        })
      }
    })?;
  }

  // Step 2: Get those arg values, and make sure they are of expected type.
  for arg in relation_args.iter() {
    match &arg.arg_name {
      Token::Ident(ident_name, _) if ident_name == ATTRIB_NAMED_ARG_NAME => {
        if let Token::String(..) = arg.arg_value {
          relation_name = Some(&arg.arg_value);
        } else {
          return Err(Error::RelationInvalidAttributeArg {
            span: arg.arg_value.span(),
            relation_name: None,
            arg_name: arg.arg_name.ident_name(),
            field_name: field.name.ident_name(),
            model_name: model.name.ident_name(),
          });
        }
      }
      Token::Ident(ident_name, _) if ident_name == ATTRIB_NAMED_ARG_FIELD => {
        relation_scalar_field = Some(get_relation_scalar_field(arg, field, model)?);
      }
      Token::Ident(ident_name, _) if ident_name == ATTRIB_NAMED_ARG_REFERENCES => {
        referenced_model_field = Some(get_referenced_model_field(
          arg,
          field,
          model,
          referenced_model,
        )?);
      }
      _ => {
        return Err(Error::RelationInvalidAttributeArg {
          span: arg.arg_value.span(),
          relation_name: None,
          arg_name: arg.arg_name.ident_name(),
          field_name: field.name.ident_name(),
          model_name: model.name.ident_name(),
        })
      }
    }
  }

  referenced_model_relation_field = get_referenced_model_relation_field(
    relation_name.expect("relation_name can't be None at this point."),
    field,
    model,
    referenced_model,
  )?;

  // Make sure relation scalar field and referenced field are of the `same primitive type`
  if relation_scalar_field.is_some()
    && referenced_model_field.is_some()
    && *relation_scalar_field.unwrap().field_type.r#type()
      != *referenced_model_field.unwrap().field_type.r#type()
  {
    Err(Error::RelationScalarAndReferencedFieldsTypeMismatch {
      span: relation_scalar_field.unwrap().name.span(),
      field_name: relation_scalar_field.unwrap().name.ident_name().unwrap(),
      model_name: model.name.ident_name().unwrap(),
      referenced_field_name: referenced_model_field.unwrap().name.ident_name().unwrap(),
      referenced_model_name: referenced_model.name.ident_name().unwrap(),
    })
  } else {
    Ok(RelationAttributeDetails {
      relation_name: relation_name.expect("relation_name can't be None at this point."),
      relation_scalar_field,
      referenced_model_field,
      referenced_model_relation_field,
    })
  }
}

fn get_relation_scalar_field<'src, 'b>(
  relation_arg_field: &'b NamedArg,
  field: &'b FieldDecl,
  model: &'b ModelDecl,
) -> Result<&'b FieldDecl, Error> {
  debug_assert!(
    matches!(&relation_arg_field.arg_name, Token::Ident(ident_name, _) if ident_name == ATTRIB_NAMED_ARG_FIELD),
    "Invalid argument passed for relation_arg_field"
  );

  let relation_scalar_field: Option<&'b FieldDecl>;
  // Validate relation scalar field.
  // Make sure relation scalar field exists in the parent model.
  if let Token::Ident(..) = relation_arg_field.arg_value {
    relation_scalar_field = model
      .fields
      .iter()
      .find(|field| field.name == relation_arg_field.arg_value);
    if relation_scalar_field.is_none() {
      Err(Error::RelationScalarFieldNotFound {
        span: relation_arg_field.arg_value.span(),
        scalar_field_name: relation_arg_field.arg_value.ident_name(),
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
      Err(Error::RelationScalarFieldIsNotPrimitive {
        span: relation_scalar_field.unwrap().name.span(),
        field_name: relation_scalar_field.unwrap().name.ident_name().unwrap(),
        model_name: model.name.ident_name().unwrap(),
      })
    } else {
      Ok(relation_scalar_field.unwrap())
    }
  } else {
    Err(Error::RelationInvalidAttributeArg {
      span: relation_arg_field.arg_value.span(),
      relation_name: None,
      arg_name: relation_arg_field.arg_name.ident_name(),
      field_name: Some(field.name.ident_name().unwrap()),
      model_name: Some(model.name.ident_name().unwrap()),
    })
  }
}

fn get_referenced_model_field<'src, 'b>(
  relation_arg_references: &'b NamedArg,
  field: &'b FieldDecl,
  model: &'b ModelDecl,
  referenced_model: &'b ModelDecl,
) -> Result<&'b FieldDecl, Error> {
  debug_assert!(
    matches!(&relation_arg_references.arg_name, Token::Ident(ident_name, _) if ident_name == ATTRIB_NAMED_ARG_REFERENCES),
    "Invalid arg is passed for relation_arg_references"
  );
  let referenced_model_field: Option<&'b FieldDecl>;
  // Validate referenced field.
  // Make sure referenced field, exists in the referenced model.
  if let Token::Ident(_, _) = relation_arg_references.arg_value {
    referenced_model_field = referenced_model
      .fields
      .iter()
      .find(|field| relation_arg_references.arg_value == field.name);
    if referenced_model_field.is_none() {
      Err(Error::RelationReferencedFieldNotFound {
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
      Err(Error::RelationReferencedFieldNotScalar {
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
        .find(|attrib| match &attrib.name {
          Token::Ident(ident_name, _) if ident_name == ATTRIB_NAME_UNIQUE => true,
          Token::Ident(ident_name, _) if ident_name == ATTRIB_NAME_ID => true,
          _ => false,
        })
        .is_none()
    }) {
      // if the referenced field is not attributed with @id or @unique then throw error.
      Err(Error::RelationReferencedFieldNotUnique {
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
    Err(Error::RelationInvalidAttributeArg {
      span: relation_arg_references.arg_value.span(),
      relation_name: None,
      arg_name: relation_arg_references.arg_name.ident_name(),
      field_name: field.name.ident_name(),
      model_name: model.name.ident_name(),
    })
  }
}

fn get_referenced_model_relation_field<'src, 'b>(
  relation_name: &'b Token,
  field: &'b FieldDecl,
  model: &'b ModelDecl,
  referenced_model: &'b ModelDecl,
) -> Result<Option<&'b FieldDecl>, Error> {
  let is_self_relation = model.name == referenced_model.name;
  let mut referenced_model_relation_field =
    referenced_model.fields.iter().filter(|fld| {
      // Does this field has relation attribute to it ?
      let mut has_relation_attribute = fld.attributes.iter().filter(|attrib| {
        match &attrib.name {
          Token::Ident(name, _) if name == ATTRIB_NAME_RELATION => {
            // In case of self relation, make sure to pick the right relation field.
            true && (!is_self_relation || fld.name != field.name)
          }
          _ => false,
        }
      });

      if let Some(relation_attrib) = has_relation_attribute.next() {
        // if yes, then does field type & relation name matches ?
        match &relation_attrib.arg {
          Some(AttribArg::Args(named_args)) => {
            named_args.iter().fold(false, |acc, named_arg| {
              if relation_name == &named_arg.arg_value
                && fld.field_type.r#type().token() == &model.name
              {
                acc || true
              } else {
                acc || false
              }
            })
          }
          _ => false,
        }
      } else {
        false
      }
    });

  Ok(referenced_model_relation_field.next())
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
