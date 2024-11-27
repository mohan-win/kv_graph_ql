use crate::graphql_parser::types::ExecutableDocument;

use super::visitor::{RuleError, Visitor};

pub(crate) fn validate<'a, V, F>(
    doc: &'a ExecutableDocument,
    factory: F,
) -> Result<(), Vec<RuleError>>
where
    V: Visitor<'a> + 'a,
    F: Fn() -> V,
{
    unimplemented!()
}

pub(crate) fn expect_pass_rule_<'a, V, F>(doc: &'a ExecutableDocument, factory: F)
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
    }
    panic!("Expected rule to pass, but errors found.");
}

macro_rules! expect_pass_rule {
    ($factory:expr, $query_source:literal $(,)?) => {
        let doc = crate::graphql_parser::parse_query($query_source).expect("Parse error");
        crate::validation::test_harness::expect_pass_rule_(&doc, $factory);
    };
}

pub(crate) fn expect_fail_rule_<'a, V, F>(doc: &'a ExecutableDocument, factory: F)
where
    V: Visitor<'a> + 'a,
    F: Fn() -> V,
{
    if validate(doc, factory).is_ok() {
        panic!("Expected rule to fail, but no errors were found");
    }
}

macro_rules! expect_fail_rule {
    ($factory:expr, $query_source:literal $(,)?) => {
        let doc = crate::graphql_parser::parse_query($query_source).expect("Parse error");
        crate::validation::test_harness::expect_fail_rule_(&doc, $factory);
    };
}
