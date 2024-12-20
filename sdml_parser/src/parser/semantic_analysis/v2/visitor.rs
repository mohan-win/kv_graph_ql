use std::collections::{HashMap, HashSet};

use crate::{
  parser::semantic_analysis::err::{self, Error},
  types::{
    Attribute, ConfigDecl, DataModel, Declaration, DeclarationsGrouped, EnumDecl,
    FieldDecl, FieldType, ModelDecl, Span,
  },
};

#[derive(Debug, Default)]
pub struct VisitorContext<'a> {
  pub errors: Vec<Error>,
  data_model: DataModel,
  current_model: Option<&'a ModelDecl>,
  current_field: Option<&'a FieldDecl>,
  current_field_relation: Option<&'a ModelDecl>,
  current_attributes: Option<&'a Vec<Attribute>>,
}

impl<'a> VisitorContext<'a> {
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

  pub fn with_current_attributes<F: FnMut(&mut VisitorContext<'a>)>(
    &mut self,
    field_attributes: &'a Vec<Attribute>,
    mut f: F,
  ) {
    assert!(
      self.current_model.is_some() && self.current_field.is_some(),
      "Field attributes can only be associated with a model field."
    );
    self.current_attributes = Some(field_attributes);
    f(self);
    self.current_attributes = None;
  }
}

pub trait Visitor<'a> {
  fn enter_declarations(
    &mut self,
    _ctx: &mut VisitorContext<'a>,
    _declarations: &'a DeclarationsGrouped,
  ) {
  }
  fn exit_declarations(
    &mut self,
    _ctx: &mut VisitorContext<'a>,
    _declarations: &'a DeclarationsGrouped,
  ) {
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

pub fn visit<'a, V: Visitor<'a>>(
  v: &mut V,
  ctx: &mut VisitorContext<'a>,
  declarations: &'a DeclarationsGrouped,
) {
  v.enter_declarations(ctx, declarations);

  // configs
  declarations.configs.values().for_each(|config| {
    v.enter_config(ctx, config);
    v.exit_config(ctx, config);
  });
  // enums
  declarations.enums.values().for_each(|r#enum| {
    v.enter_enum(ctx, r#enum);
    v.exit_enum(ctx, r#enum);
  });

  // Models
  declarations.models.values().for_each(|model| {
    visit_model(v, ctx, declarations, model);
  });

  v.exit_declarations(ctx, declarations);
}

fn visit_model<'a, V: Visitor<'a>>(
  v: &mut V,
  ctx: &mut VisitorContext<'a>,
  declarations: &'a DeclarationsGrouped,
  model: &'a ModelDecl,
) {
  v.enter_model(ctx, model);

  model.fields.iter().for_each(|field| {
    ctx.with_model(model, |ctx| {
      visit_field(v, ctx, declarations, field);
    });
  });

  v.exit_model(ctx, model);
}

fn visit_field<'a, V: Visitor<'a>>(
  v: &mut V,
  ctx: &mut VisitorContext<'a>,
  declarations: &'a DeclarationsGrouped,
  field: &'a FieldDecl,
) {
  v.enter_field(ctx, field);

  let type_name = field
    .field_type
    .r#type()
    .token()
    .ident_name()
    .expect("Field type should be an identifier");
  let field_relation = declarations.models.get(&type_name);
  ctx.with_field_relation(field_relation, |ctx| {
    ctx.with_current_attributes(&field.attributes, |ctx| {
      field.attributes.iter().for_each(|attribute| {
        v.enter_attribute(ctx, attribute);
        v.exit_attribute(ctx, attribute);
      });
    })
  });

  v.exit_field(ctx, field);
}

pub fn categorise_declarations(
  declarations: Vec<Declaration>,
) -> Result<DeclarationsGrouped, Vec<Error>> {
  let mut errs: Vec<Error> = Vec::new();
  let mut type_set: HashSet<String> = HashSet::new();

  let mut configs = HashMap::new();
  let mut enums = HashMap::new();
  let mut models = HashMap::new();

  for decl in declarations.into_iter() {
    let (type_name, span) = match decl {
      Declaration::Config(c) => {
        let type_name = c.name.ident_name().unwrap();
        let span = c.name.span();
        configs.insert(type_name.clone(), c);
        (type_name, span)
      }
      Declaration::Enum(e) => {
        let type_name = e.name.ident_name().unwrap();
        let span = e.name.span();
        enums.insert(type_name.clone(), e);
        (type_name, span)
      }
      Declaration::Model(m) => {
        let type_name = m.name.ident_name().unwrap();
        let span = m.name.span();
        models.insert(type_name.clone(), m);
        (type_name, span)
      }
    };

    if type_set.contains(&type_name) {
      errs.push(Error::TypeDuplicateDefinition { span, type_name });
    } else {
      type_set.insert(type_name);
    }
  }

  if errs.is_empty() {
    Ok(DeclarationsGrouped {
      configs,
      enums,
      models,
    })
  } else {
    Err(errs)
  }
}
