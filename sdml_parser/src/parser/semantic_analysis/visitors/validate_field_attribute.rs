use std::collections::HashMap;

use crate::{
  parser::semantic_analysis::{
    attribute::{AllowedFieldType, AttributeDetails},
    err::Error,
    visitor::{Visitor, VisitorMode},
  },
  types::{Attribute, FieldDecl, ModelDecl},
};

/// Validates if the attribute can be applied to the
/// given field.
pub struct ValidateFieldAttribute;

impl<'a> Visitor<'a> for ValidateFieldAttribute {
  fn enter_data_model(
    &mut self,
    ctx: &mut super::VisitorContext<'a>,
    _data_model: &'a crate::types::DataModel,
  ) {
    assert!(
      matches!(ctx.mode(), VisitorMode::Validate(_)),
      "This visitor is valid only on `Validate` mode."
    );
  }
  fn enter_attribute(
    &mut self,
    ctx: &mut super::VisitorContext<'a>,
    attribute: &'a crate::types::Attribute,
  ) {
    let _ = Self::is_valid_field_type(
      attribute,
      &ctx.current_field.unwrap(),
      &ctx.current_model.unwrap(),
      &ctx.ATTRIBUTE_DETAILS_MAP,
    )
    .map_err(|err| {
      ctx.report_error(err);
    });
  }
}

impl ValidateFieldAttribute {
  /// Is the field_type valid for the given attribute ?
  fn is_valid_field_type(
    attrib: &Attribute,
    field: &FieldDecl,
    model: &ModelDecl,
    attribute_details_map: &HashMap<&'static str, AttributeDetails>,
  ) -> Result<(), Error> {
    match attribute_details_map.get(attrib.name.ident_name().unwrap().as_str()) {
      None => Err(Error::AttributeUnknown {
        span: attrib.name.span(),
        attrib_name: attrib.name.ident_name().unwrap(),
        field_name: field.name.ident_name().unwrap(),
        model_name: model.name.ident_name().unwrap(),
      }),
      Some(attrib_detail) => {
        let is_scalar_short_str_field = field.field_type.is_scalar_short_str();
        let is_scalar_field = field.field_type.is_scalar();
        let is_optional_field = field.field_type.is_optional();

        let invalid_attribute_err = Err(Error::AttributeInvalid {
          span: attrib.name.span(),
          reason: attrib_detail.allowed_field_type.to_string(),
          attrib_name: attrib.name.ident_name().unwrap(),
          field_name: field.name.ident_name().unwrap(),
          model_name: model.name.ident_name().unwrap(),
        });
        match attrib_detail.allowed_field_type {
          AllowedFieldType::ScalarShortStrField { can_be_optional }
            if is_scalar_short_str_field =>
          {
            if !can_be_optional && is_optional_field {
              invalid_attribute_err
            } else {
              Ok(())
            }
          }
          AllowedFieldType::ScalarField { can_be_optional } if is_scalar_field => {
            if !can_be_optional && is_optional_field {
              invalid_attribute_err
            } else {
              Ok(())
            }
          }
          AllowedFieldType::NonScalarField { can_be_optional } if !is_scalar_field => {
            if !can_be_optional && is_optional_field {
              invalid_attribute_err
            } else {
              Ok(())
            }
          }
          AllowedFieldType::AnyField { can_be_optional } => {
            if !can_be_optional && is_optional_field {
              invalid_attribute_err
            } else {
              Ok(())
            }
          }
          _ => invalid_attribute_err,
        }
      }
    }
  }
}
