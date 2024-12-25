use std::collections::HashMap;

use crate::{
  parser::semantic_analysis::{
    attribute::{AttributeDetails, ATTRIB_ARG_VALUE_ENUM},
    err::Error,
    visitor::{Visitor, VisitorMode},
  },
  types::{AttribArg, Attribute, EnumDecl, FieldDecl, ModelDecl, Type},
};

/// Validate the attribute arguments
pub struct ValidateAttributeArgs;

impl<'a> Visitor<'a> for ValidateAttributeArgs {
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
    let _ = Self::validate_attribute_args(
      attribute,
      &ctx.current_field.unwrap(),
      &ctx.current_model.unwrap(),
      &ctx.input_enums(),
      &ctx.ATTRIBUTE_DETAILS_MAP,
    )
    .map_err(|err| {
      ctx.report_error(err);
    });
  }
}

impl ValidateAttributeArgs {
  fn validate_attribute_args(
    attrib: &Attribute,
    field: &FieldDecl,
    model: &ModelDecl,
    enums: &HashMap<String, EnumDecl>,
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
        if attrib
          .arg
          .as_ref()
          .is_some_and(|_| attrib_detail.is_empty_arg_attribute())
        {
          // Is this an empty arg attribute incorrectly having some arguments ?
          Err(Error::AttributeArgInvalid {
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
                .contains(&fn_name.ident_name().unwrap().as_str())
              {
                Err(Error::AttributeArgInvalid {
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
                  Err(Error::AttributeArgInvalid {
                    span: arg_value.span(),
                    attrib_arg_name: Some(arg_value.ident_name().unwrap()),
                    attrib_name: attrib.name.ident_name().unwrap(),
                    field_name: field.name.ident_name().unwrap(),
                    model_name: model.name.ident_name().unwrap(),
                  })
                } else {
                  let enum_decl = enums
                    .get(enum_ty_name.ident_name().unwrap().as_str())
                    .ok_or_else(|| Error::TypeUndefined {
                      span: enum_ty_name.span(),
                      type_name: enum_ty_name.ident_name().unwrap(),
                      field_name: field.name.ident_name().unwrap(),
                      model_name: model.name.ident_name().unwrap(),
                    })?;
                  if enum_decl.elements.contains(arg_value) {
                    Ok(())
                  } else {
                    Err(Error::EnumValueUndefined {
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
                .contains(&arg_value.ident_name().unwrap().as_str())
              {
                Err(Error::AttributeArgInvalid {
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
                  .contains(&named_arg.arg_name.ident_name().unwrap().as_str())
                {
                  Some(&named_arg.arg_name)
                } else {
                  None
                }
              });
              if let Some(invalid_arg) = invalid_args.next() {
                Err(Error::AttributeArgInvalid {
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
}
