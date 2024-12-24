use crate::{
  parser::semantic_analysis::{err::Error, visitor::Visitor},
  types::{ModelDecl, ModelIndexedFieldsFilter},
};

pub struct ValidateModelHasIdField;

impl<'a> Visitor<'a> for ValidateModelHasIdField {
  fn enter_model(
    &mut self,
    ctx: &mut super::VisitorContext<'a>,
    model: &'a crate::types::ModelDecl,
  ) {
    let _ = Self::validate_model_id_field(model).map_err(|err| {
      ctx.report_error(err);
    });
  }
}

impl ValidateModelHasIdField {
  fn validate_model_id_field(model: &ModelDecl) -> Result<(), Error> {
    let model_fields = model.get_fields_internal(true); // Note: allow_unknown_field_type is set to `true`. Because this function is called during the semantic_update phase.
    let has_only_auto_gen_id = model_fields
      .id
      .iter()
      .fold(true, |acc, (_id_fld, is_auto_gen)| acc && *is_auto_gen);
    let is_empty_model = model_fields
      .get_rest(ModelIndexedFieldsFilter::All)
      .is_empty()
      && model_fields.unique.is_empty()
      && has_only_auto_gen_id;

    if is_empty_model {
      Err(Error::ModelEmpty {
        span: model.name.span(),
        model_name: model.name.ident_name().unwrap(),
      })
    } else if model_fields.id.is_empty() {
      Err(Error::ModelIdFieldMissing {
        span: model.name.span(),
        model_name: model.name.ident_name().unwrap(),
      })
    } else if model_fields.id.len() > 1 {
      let (second_id_field, _) = model_fields.id[1];
      // Is there more than one Id field in a Model ?
      Err(Error::ModelIdFieldDuplicate {
        span: second_id_field.name.span(),
        field_name: second_id_field.name.ident_name().unwrap(),
        model_name: model.name.ident_name().unwrap(),
      })
    } else {
      Ok(())
    }
  }
}
