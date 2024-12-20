use crate::{
  parser::semantic_analysis::err::Error,
  types::{DataModel, FieldDecl, ModelDecl},
};

use super::visitor::{VisitMode, Visitor, VisitorContext};

pub fn validate_data_model<'a, V: Visitor<'a>>(
  v: &mut V,
  data_model: &'a DataModel,
) -> Result<(), Vec<Error>> {
  let mut ctx = VisitorContext::new(VisitMode::Validation(&data_model));

  v.enter_data_model(&mut ctx, data_model);

  // configs
  data_model.configs.values().for_each(|config| {
    v.enter_config(&mut ctx, config);
    v.exit_config(&mut ctx, config);
  });
  // enums
  data_model.enums.values().for_each(|r#enum| {
    v.enter_enum(&mut ctx, r#enum);
    v.exit_enum(&mut ctx, r#enum);
  });

  // Models
  data_model.models.values().for_each(|model| {
    visit_model_for_validation(v, &mut ctx, model);
  });

  v.exit_data_model(&mut ctx, data_model);

  if ctx.errors.is_empty() {
    Ok(())
  } else {
    Err(ctx.errors)
  }
}

fn visit_model_for_validation<'a, V: Visitor<'a>>(
  v: &mut V,
  ctx: &mut VisitorContext<'a>,
  model: &'a ModelDecl,
) {
  v.enter_model(ctx, model);

  model.fields.iter().for_each(|field| {
    ctx.with_model(model, |ctx| {
      visit_field_for_validation(v, ctx, field);
    });
  });

  v.exit_model(ctx, model);
}

fn visit_field_for_validation<'a, V: Visitor<'a>>(
  v: &mut V,
  ctx: &mut VisitorContext<'a>,
  field: &'a FieldDecl,
) {
  v.enter_field(ctx, field);

  let type_name = field
    .field_type
    .r#type()
    .token()
    .ident_name()
    .expect("Field type should be an identifier");
  let field_relation = ctx
    .input_data_model()
    .expect("Data Model should be present for validation")
    .models
    .get(&type_name);
  ctx.with_field_relation(field_relation, |ctx| {
    field.attributes.iter().for_each(|attribute| {
      ctx.with_current_attribute(attribute, |ctx| {
        v.enter_attribute(ctx, attribute);
        v.exit_attribute(ctx, attribute);
      });
    });
  });

  v.exit_field(ctx, field);
}
