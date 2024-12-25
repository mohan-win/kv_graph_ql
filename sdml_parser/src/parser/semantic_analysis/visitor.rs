//! Module for both semantic analysis
//! Not that

use std::collections::HashMap;

use crate::{
  parser::semantic_analysis::{attribute::AttributeDetails, err::Error, RelationMap},
  types::{
    Attribute, ConfigDecl, DataModel, DeclarationsGrouped, EnumDecl, FieldDecl,
    ModelDecl, Type,
  },
};

pub(crate) enum VisitorMode<'a> {
  /// Visitor mode for building the DataModel from the given declarations.
  Build(&'a DeclarationsGrouped),
  /// Visitor mode for validating the given data model.
  Validate(&'a DataModel),
}

pub(crate) struct VisitorContext<'a> {
  mode: VisitorMode<'a>,
  attribute_details_map: HashMap<&'static str, AttributeDetails>,
  pub errors: Vec<Error>,
  /// Newly built `data_model`.
  built_data_model: DataModel,
  /// Result of VisitMode::Update.
  pub(crate) current_model: Option<&'a ModelDecl>,
  pub(crate) current_field: Option<&'a FieldDecl>,
  pub(crate) current_field_relation: Option<&'a ModelDecl>,
  pub(crate) current_attribute: Option<&'a Attribute>,
}

impl<'a> VisitorContext<'a> {
  pub fn new(mode: VisitorMode<'a>) -> VisitorContext<'a> {
    let built_data_model = match mode {
      VisitorMode::Build(declarations) => declarations.clone().into(), // Note: starting the data model with existing declarations.
      VisitorMode::Validate(_) => DataModel::default(),
    };
    Self {
      mode,
      attribute_details_map: AttributeDetails::attributes_detail_map(),
      errors: Default::default(),
      built_data_model,
      current_model: None,
      current_field: None,
      current_field_relation: None,
      current_attribute: None,
    }
  }

  pub fn mode(&self) -> &VisitorMode<'a> {
    &self.mode
  }

  pub fn input_models(&self) -> &'a HashMap<String, ModelDecl> {
    match self.mode {
      VisitorMode::Build(declarations) => &declarations.models,
      VisitorMode::Validate(data_model) => &data_model.models,
    }
  }

  pub fn input_enums(&self) -> &'a HashMap<String, EnumDecl> {
    match self.mode {
      VisitorMode::Build(declarations) => &declarations.enums,
      VisitorMode::Validate(data_model) => &data_model.enums,
    }
  }

  #[allow(dead_code)]
  pub fn input_configs(&self) -> &'a HashMap<String, ConfigDecl> {
    match self.mode {
      VisitorMode::Build(declarations) => &declarations.configs,
      VisitorMode::Validate(data_model) => &data_model.configs,
    }
  }

  pub fn attribute_details_map(&self) -> &HashMap<&'static str, AttributeDetails> {
    &self.attribute_details_map
  }

  /// Retrieve the newly built data model.
  pub fn data_model(self) -> DataModel {
    assert!(
      matches!(self.mode, VisitorMode::Build(_)),
      "Data Model is build only on `VisitorMode::Build`"
    );
    self.built_data_model
  }

  /// Update the field type of the current_field, and return the updated field.
  /// ### Note:
  /// The update happens in the `updated_data_model`.
  /// (i.e) The `input_declarations` never touched.
  pub fn update_current_field_type(&mut self, actual_type: Type) {
    // get the current field.
    let model_to_update = self
      .built_data_model
      .models
      .get_mut(&self.current_model.unwrap().name.ident_name().unwrap());
    let field_to_update = model_to_update
      .unwrap()
      .field_by_name_mut(&self.current_field.unwrap().name.ident_name().unwrap())
      .expect("Current Field should present inside the current model");

    debug_assert!(
      matches!(field_to_update.field_type.r#type(), Type::Unknown(..)),
      "Only fields with unknown types should be updated!"
    );
    field_to_update.field_type.set_type(actual_type);
  }

  /// Updates the relations in `updated_data_model`.
  pub fn update_relation_map(&mut self, relations: RelationMap) {
    let _ = relations
      .get_valid_relations()
      .map(|valid_relations| self.built_data_model.relations = valid_relations)
      .is_err_and(|errs| {
        // Report errs if any!
        self.append_errors(errs);
        true
      });
  }

  pub fn report_error(&mut self, err: Error) {
    self.errors.push(err);
  }

  pub fn append_errors(&mut self, errors: Vec<Error>) {
    self.errors.extend(errors);
  }

  pub fn with_model<F: FnMut(&mut VisitorContext<'a>)>(
    &mut self,
    model: &'a ModelDecl,
    mut f: F,
  ) {
    self.current_model = Some(model);
    f(self);
    self.current_model = None
  }

  pub fn with_field<F: FnMut(&mut VisitorContext<'a>)>(
    &mut self,
    field: &'a FieldDecl,
    mut f: F,
  ) {
    assert!(
      self.current_model.is_some(),
      "Field can not exist outside model"
    );
    self.current_field = Some(field);
    f(self);
    self.current_field = None;
  }

  pub fn with_field_relation<F: FnMut(&mut VisitorContext<'a>)>(
    &mut self,
    field_relation: Option<&'a ModelDecl>,
    mut f: F,
  ) {
    assert!(
      self.current_model.is_some() && self.current_field.is_some(),
      "Only a model field can represent a relation."
    );
    self.current_field_relation = field_relation;
    f(self);
    self.current_field_relation = None;
  }

  pub fn with_current_attribute<F: FnMut(&mut VisitorContext<'a>)>(
    &mut self,
    field_attribute: &'a Attribute,
    mut f: F,
  ) {
    assert!(
      self.current_model.is_some() && self.current_field.is_some(),
      "Field attribute can only be associated with a model field."
    );
    self.current_attribute = Some(field_attribute);
    f(self);
    self.current_attribute = None;
  }
}

pub trait Visitor<'a> {
  fn enter_declarations(
    &mut self,
    ctx: &mut VisitorContext<'a>,
    _declarations: &'a DeclarationsGrouped,
  ) {
    assert!(
      matches!(ctx.mode, VisitorMode::Build(_)),
      "Should be called only on `VisitorMode::Build`"
    )
  }
  fn exit_declarations(
    &mut self,
    ctx: &mut VisitorContext<'a>,
    _declarations: &'a DeclarationsGrouped,
  ) {
    assert!(
      matches!(ctx.mode, VisitorMode::Build(_)),
      "Should be called only on `VisitorMode::Build`"
    )
  }

  fn enter_data_model(
    &mut self,
    ctx: &mut VisitorContext<'a>,
    _data_model: &'a DataModel,
  ) {
    assert!(
      matches!(ctx.mode, VisitorMode::Validate(_)),
      "Should be called only on `VisitorMode::Validate`"
    )
  }
  fn exit_data_model(
    &mut self,
    ctx: &mut VisitorContext<'a>,
    _data_model: &'a DataModel,
  ) {
    assert!(
      matches!(ctx.mode, VisitorMode::Validate(_)),
      "Should be called only on `VisitorMode::Validate`"
    )
  }

  fn enter_config(&mut self, _ctx: &mut VisitorContext<'a>, _config: &'a ConfigDecl) {}
  fn exit_config(&mut self, _ctx: &mut VisitorContext<'a>, _config: &'a ConfigDecl) {}

  fn enter_enum(&mut self, _ctx: &mut VisitorContext<'a>, _enum: &'a EnumDecl) {}
  fn exit_enum(&mut self, _ctx: &mut VisitorContext<'a>, _enum: &'a EnumDecl) {}

  fn enter_model(&mut self, _ctx: &mut VisitorContext<'a>, _model: &'a ModelDecl) {}
  fn exit_model(&mut self, _ctx: &mut VisitorContext<'a>, _model: &'a ModelDecl) {}

  fn enter_field(&mut self, _ctx: &mut VisitorContext<'a>, _field: &'a FieldDecl) {}
  fn exit_field(&mut self, _ctx: &mut VisitorContext<'a>, _field: &'a FieldDecl) {}

  fn enter_attribute(
    &mut self,
    _ctx: &mut VisitorContext<'a>,
    _attribute: &'a Attribute,
  ) {
  }
  fn exit_attribute(&mut self, _ctx: &mut VisitorContext<'a>, _attribute: &'a Attribute) {
  }
}

pub struct VisitorNil;

impl VisitorNil {
  pub fn with<V>(self, visitor: V) -> VisitorCons<V, Self> {
    VisitorCons(visitor, self)
  }
}

pub struct VisitorCons<A, B>(A, B);

impl<A, B> VisitorCons<A, B> {
  pub const fn with<V>(self, visitor: V) -> VisitorCons<V, Self> {
    VisitorCons(visitor, self)
  }
}

impl Visitor<'_> for VisitorNil {}

impl<'a, A, B> Visitor<'a> for VisitorCons<A, B>
where
  A: Visitor<'a> + 'a,
  B: Visitor<'a> + 'a,
{
  fn enter_declarations(
    &mut self,
    ctx: &mut VisitorContext<'a>,
    declarations: &'a DeclarationsGrouped,
  ) {
    self.0.enter_declarations(ctx, declarations);
    self.1.enter_declarations(ctx, declarations);
  }
  fn exit_declarations(
    &mut self,
    ctx: &mut VisitorContext<'a>,
    declarations: &'a DeclarationsGrouped,
  ) {
    self.0.exit_declarations(ctx, declarations);
    self.1.exit_declarations(ctx, declarations);
  }

  fn enter_data_model(
    &mut self,
    ctx: &mut VisitorContext<'a>,
    data_model: &'a DataModel,
  ) {
    self.0.enter_data_model(ctx, data_model);
    self.1.enter_data_model(ctx, data_model);
  }
  fn exit_data_model(&mut self, ctx: &mut VisitorContext<'a>, data_model: &'a DataModel) {
    self.0.exit_data_model(ctx, data_model);
    self.1.exit_data_model(ctx, data_model);
  }

  fn enter_config(&mut self, ctx: &mut VisitorContext<'a>, config: &'a ConfigDecl) {
    self.0.enter_config(ctx, config);
    self.1.enter_config(ctx, config);
  }
  fn exit_config(&mut self, ctx: &mut VisitorContext<'a>, config: &'a ConfigDecl) {
    self.0.exit_config(ctx, config);
    self.1.exit_config(ctx, config);
  }

  fn enter_enum(&mut self, ctx: &mut VisitorContext<'a>, r#enum: &'a EnumDecl) {
    self.0.enter_enum(ctx, r#enum);
    self.1.enter_enum(ctx, r#enum);
  }
  fn exit_enum(&mut self, ctx: &mut VisitorContext<'a>, r#enum: &'a EnumDecl) {
    self.0.exit_enum(ctx, r#enum);
    self.1.exit_enum(ctx, r#enum);
  }

  fn enter_model(&mut self, ctx: &mut VisitorContext<'a>, model: &'a ModelDecl) {
    self.0.enter_model(ctx, model);
    self.1.enter_model(ctx, model);
  }
  fn exit_model(&mut self, ctx: &mut VisitorContext<'a>, model: &'a ModelDecl) {
    self.0.exit_model(ctx, model);
    self.1.exit_model(ctx, model);
  }

  fn enter_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a FieldDecl) {
    self.0.enter_field(ctx, field);
    self.1.enter_field(ctx, field);
  }
  fn exit_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a FieldDecl) {
    self.0.exit_field(ctx, field);
    self.1.exit_field(ctx, field);
  }

  fn enter_attribute(&mut self, ctx: &mut VisitorContext<'a>, attribute: &'a Attribute) {
    self.0.enter_attribute(ctx, attribute);
    self.1.enter_attribute(ctx, attribute);
  }
  fn exit_attribute(&mut self, ctx: &mut VisitorContext<'a>, attribute: &'a Attribute) {
    self.0.exit_attribute(ctx, attribute);
    self.1.exit_attribute(ctx, attribute);
  }
}

/// Builds data_model from the given declarations.
pub fn build_data_model<'a, V: Visitor<'a>>(
  v: &mut V,
  declarations: &'a DeclarationsGrouped,
) -> Result<DataModel, Vec<Error>> {
  let mut ctx = VisitorContext::new(VisitorMode::Build(declarations));

  v.enter_declarations(&mut ctx, declarations);

  // configs
  declarations.configs.values().for_each(|config| {
    v.enter_config(&mut ctx, config);
    v.exit_config(&mut ctx, config);
  });
  // enums
  declarations.enums.values().for_each(|r#enum| {
    v.enter_enum(&mut ctx, r#enum);
    v.exit_enum(&mut ctx, r#enum);
  });

  // Models
  declarations.models.values().for_each(|model| {
    ctx.with_model(model, |ctx| {
      visit_model(v, ctx, model);
    });
  });

  v.exit_declarations(&mut ctx, declarations);

  if ctx.errors.is_empty() {
    Ok(ctx.data_model())
  } else {
    Err(ctx.errors)
  }
}

/// Validates the `data_model`.
pub fn validate_data_model<'a, V: Visitor<'a>>(
  v: &mut V,
  data_model: &'a DataModel,
) -> Result<(), Vec<Error>> {
  let mut ctx = VisitorContext::new(VisitorMode::Validate(data_model));

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
    ctx.with_model(model, |ctx| {
      visit_model(v, ctx, model);
    });
  });

  v.exit_data_model(&mut ctx, data_model);

  if ctx.errors.is_empty() {
    Ok(())
  } else {
    Err(ctx.errors)
  }
}

fn visit_model<'a, V: Visitor<'a>>(
  v: &mut V,
  ctx: &mut VisitorContext<'a>,
  model: &'a ModelDecl,
) {
  v.enter_model(ctx, model);

  model.fields.iter().for_each(|field| {
    ctx.with_field(field, |ctx| {
      let type_name = field
        .field_type
        .r#type()
        .token()
        .ident_name()
        .expect("Field type should be an identifier");
      let field_relation = ctx.input_models().get(&type_name);
      ctx.with_field_relation(field_relation, |ctx| {
        visit_field(v, ctx, field);
      });
    });
  });

  v.exit_model(ctx, model);
}

fn visit_field<'a, V: Visitor<'a>>(
  v: &mut V,
  ctx: &mut VisitorContext<'a>,
  field: &'a FieldDecl,
) {
  v.enter_field(ctx, field);

  field.attributes.iter().for_each(|attribute| {
    ctx.with_current_attribute(attribute, |ctx| {
      v.enter_attribute(ctx, attribute);
      v.exit_attribute(ctx, attribute);
    });
  });

  v.exit_field(ctx, field);
}
