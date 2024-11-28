use crate::{
  graphql_parser::{self, types::ExecutableDocument},
  registry::Registry,
};

use super::visitor::{self, visit, RuleError, Visitor, VisitorContext};
use once_cell::sync::Lazy;
use std::fs;

fn build_registry(
  schema_file_path: &str,
) -> Result<Registry, Box<dyn std::error::Error>> {
  let schema_file = fs::read_to_string(schema_file_path)?;
  let service_doc = graphql_parser::parse_schema(schema_file)?;
  let registry = Registry::build_registry(service_doc);
  Ok(registry)
}

static REGISTRY: Lazy<Registry> = Lazy::new(|| {
  build_registry(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/test_data/validation/test_schema.graphql"
  ))
  .expect("Unable to build registry")
});

pub(crate) fn validate<'a, V, F>(
  doc: &'a ExecutableDocument,
  factory: F,
) -> Result<(), Vec<RuleError>>
where
  V: Visitor<'a> + 'a,
  F: Fn() -> V,
{
  let mut ctx = VisitorContext::new(&*REGISTRY, doc, None);
  let mut visitor = factory();
  visit(&mut visitor, &mut ctx, doc);
  if ctx.errors.is_empty() {
    Ok(())
  } else {
    Err(ctx.errors)
  }
}

pub(crate) fn expect_passes_rule_<'a, V, F>(doc: &'a ExecutableDocument, factory: F)
where
  V: Visitor<'a> + 'a,
  F: Fn() -> V,
{
  if let Err(errors) = validate(doc, factory) {
    for err in errors {
      if let Some(position) = err.locations.first() {
        print!("[{}:{}]", position.line, position.column);
      }
      println!("{}", err.message);
    }
    panic!("Expected rule to pass, but errors found.");
  }
}

macro_rules! expect_passes_rule {
  ($factory:expr, $query_source:literal $(,)?) => {
    let doc = crate::graphql_parser::parse_query($query_source).expect("Parse error");
    crate::validation::test_harness::expect_passes_rule_(&doc, $factory);
  };
}

pub(crate) fn expect_fails_rule_<'a, V, F>(doc: &'a ExecutableDocument, factory: F)
where
  V: Visitor<'a> + 'a,
  F: Fn() -> V,
{
  if validate(doc, factory).is_ok() {
    panic!("Expected rule to fail, but no errors were found");
  }
}

macro_rules! expect_fails_rule {
  ($factory:expr, $query_source:literal $(,)?) => {
    let doc = crate::graphql_parser::parse_query($query_source).expect("Parse error");
    crate::validation::test_harness::expect_fails_rule_(&doc, $factory);
  };
}
