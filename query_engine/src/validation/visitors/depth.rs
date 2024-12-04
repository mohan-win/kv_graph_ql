use crate::{
  graphql_parser::{types::Field, Positioned},
  validation::visitor::{VisitMode, Visitor, VisitorContext},
};

pub struct DepthCalculate<'a> {
  max_depth: &'a mut usize,
  current_depth: usize,
}

impl<'a> DepthCalculate<'a> {
  pub fn new(max_depth: &'a mut usize) -> Self {
    Self {
      max_depth,
      current_depth: 0,
    }
  }
}

impl<'ctx, 'a> Visitor<'ctx> for DepthCalculate<'a> {
  fn mode(&self) -> VisitMode {
    VisitMode::Inline
  }

  fn enter_field(
    &mut self,
    _ctx: &mut VisitorContext<'ctx>,
    _field: &'ctx Positioned<Field>,
  ) {
    self.current_depth += 1;
    *self.max_depth = (*self.max_depth).max(self.current_depth)
  }

  fn exit_field(
    &mut self,
    _ctx: &mut VisitorContext<'ctx>,
    _field: &'ctx Positioned<Field>,
  ) {
    self.current_depth -= 1;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    graphql_parser::parse_query,
    validation::{test_harness::build_registry, visitor::visit},
  };

  fn check_depth(query: &str, expect_depth: usize) {
    let registry = build_registry(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/validation/test_depth.graphql"
    ))
    .expect("Unable to build registry");
    let doc = parse_query(query).unwrap();
    let mut ctx = VisitorContext::new(&registry, &doc, None);
    let mut depth = 0;
    let mut depth_calculate = DepthCalculate::new(&mut depth);
    visit(&mut depth_calculate, &mut ctx, &doc);
    assert_eq!(depth, expect_depth);
  }

  #[test]
  fn depth() {
    check_depth(
      r#"{
          value #1
      }"#,
      1,
    );

    check_depth(
      r#"
      {
          obj { #1
              a b #2
          }
      }"#,
      2,
    );

    check_depth(
      r#"
      {
          obj { # 1
              a b c { # 2
                  a b c { # 3
                      a b # 4
                  }
              }
          }
      }"#,
      4,
    );

    check_depth(
      r#"
      fragment A on MyObj {
          a b ... A2 #2
      }
      
      fragment A2 on MyObj {
          obj {
              a #3
          }
      }
          
      query {
          obj { # 1
              ... A
          }
      }"#,
      3,
    );

    check_depth(
      r#"
      {
          obj { # 1
              ... on MyObj {
                  a b #2 
                  ... on MyObj {
                      obj {
                          a #3
                      }
                  }
              }
          }
      }"#,
      3,
    );
  }
}
