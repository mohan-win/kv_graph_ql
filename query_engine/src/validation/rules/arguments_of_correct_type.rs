use indexmap::IndexMap;

use crate::{
  context::{QueryPathNode, QueryPathSegment},
  graphql_parser::{
    types::{Directive, Field},
    Positioned,
  },
  registry::MetaInputValue,
  validation::{
    utlis::is_valid_input_value,
    visitor::{Visitor, VisitorContext},
  },
};

#[derive(Default)]
pub struct ArgumentsOfCorrectType<'a> {
  current_args: Option<&'a IndexMap<String, MetaInputValue>>,
}

impl<'a> Visitor<'a> for ArgumentsOfCorrectType<'a> {
  fn enter_directive(
    &mut self,
    ctx: &mut VisitorContext<'a>,
    directive: &'a Positioned<Directive>,
  ) {
    self.current_args = ctx
      .registry
      .directives
      .get(directive.node.name.node.as_str())
      .map(|d| &d.args)
  }

  fn exit_directive(
    &mut self,
    _ctx: &mut VisitorContext<'a>,
    _directive: &'a Positioned<Directive>,
  ) {
    self.current_args = None
  }

  fn enter_argument(
    &mut self,
    ctx: &mut VisitorContext<'a>,
    name: &'a Positioned<graphql_value::Name>,
    value: &'a Positioned<graphql_value::Value>,
  ) {
    if let Some(arg) = self
      .current_args
      .and_then(|args| args.get(name.node.as_str()))
    {
      let value = value
        .node
        .clone()
        .into_const_with(|var_name| {
          ctx
            .variables
            .and_then(|variables| variables.get(&var_name))
            .cloned()
            .ok_or(())
        })
        .ok();
      if let Some(reason) = value.and_then(|value| {
        is_valid_input_value(
          ctx.registry,
          &arg.ty,
          &value,
          QueryPathNode {
            parent: None,
            segment: QueryPathSegment::Name(&arg.name),
          },
        )
      }) {
        ctx.report_error(
          vec![name.pos],
          format!("Invalid value for argument {}", reason),
        );
      }
    }
  }

  fn enter_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a Positioned<Field>) {
    self.current_args = ctx
      .parent_type()
      .and_then(|p| p.field_by_name(&field.node.name.node))
      .map(|f| &f.args)
  }
  fn exit_field(&mut self, _ctx: &mut VisitorContext<'a>, _field: &'a Positioned<Field>) {
    self.current_args = None;
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::validation;

  pub fn factory<'a>() -> ArgumentsOfCorrectType<'a> {
    ArgumentsOfCorrectType::default()
  }

  #[test]
  fn good_null_value() {
    expect_pass_rule!(
      factory,
      r#"
        {
            complicatedArgs {
                intArgField(intArg: null)
            }
        }
        "#,
    );
  }

  #[test]
  fn null_into_int() {
    expect_fail_rule!(
      factory,
      r#"
        {
            complicatedArgs {
                nonNullIntArgField(nonNullIntArg: null)
            }
        }
        "#
    );
  }
}
