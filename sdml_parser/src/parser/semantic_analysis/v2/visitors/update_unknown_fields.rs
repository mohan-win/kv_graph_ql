use super::*;

#[derive(Debug, Default)]
pub struct UpdateUnknownFields {}

impl<'a> Visitor<'a> for UpdateUnknownFields {
  fn enter_field(&mut self, _ctx: &mut VisitorContext<'a>, _field: &'a FieldDecl) {
    unimplemented!()
  }
}
