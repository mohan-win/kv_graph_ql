use std::collections::HashMap;

use crate::{
  parser::semantic_analysis::{
    attribute::AttributeDetails, err::Error, visitor::Visitor,
  },
  types::{FieldDecl, ModelDecl},
};

/// Validates if the attributes present a field are compatible with each other, so that
/// they can be appied to a single field.
pub struct ValidateFieldAttributes;

impl<'a> Visitor<'a> for ValidateFieldAttributes {
  fn enter_field(
    &mut self,
    ctx: &mut super::VisitorContext<'a>,
    field: &'a crate::types::FieldDecl,
  ) {
    let _ = Self::is_attributes_compatible(
      field,
      &ctx.current_model.unwrap(),
      ctx.attribute_details_map(),
    )
    .map_err(|err| {
      ctx.report_error(err);
    });
  }
}

impl ValidateFieldAttributes {
  fn is_attributes_compatible(
    field: &FieldDecl,
    model: &ModelDecl,
    attribute_details_map: &HashMap<&'static str, AttributeDetails>,
  ) -> Result<(), Error> {
    if field.attributes.len() == 0 {
      Ok(())
    } else {
      let first_attribute = &field.attributes[0];
      match attribute_details_map.get(first_attribute.name.ident_name().unwrap().as_str())
      {
        Some(attribute_detail) => {
          if field.attributes.len() > 1 {
            field.attributes[1..].iter().try_for_each(|attribute| {
              if !attribute_detail
                .compatible_attribute_names
                .contains(&attribute.name.ident_name().unwrap().as_str())
              {
                Err(Error::AttributeIncompatible {
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
        None => Err(Error::AttributeUnknown {
          span: first_attribute.name.span(),
          attrib_name: first_attribute.name.ident_name().unwrap(),
          field_name: field.name.ident_name().unwrap(),
          model_name: model.name.ident_name().unwrap(),
        }),
      }
    }
  }
}
