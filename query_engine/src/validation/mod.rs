#[cfg(test)]
#[macro_use]
mod test_harness;
mod rules;
mod suggestion;
mod utils;
mod visitor;
mod visitors;

use crate::graphql_parser::types::ExecutableDocument;
use visitor::{visit, VisitorContext, VisitorNil};

use crate::{error::ServerError, graphql_value::Variables, registry::Registry};

#[derive(Debug, Clone, Copy)]
pub struct ValidationResult {
  /// Query depth.
  pub depth: usize,
}

pub enum ValidationMode {
  /// Executes all validation rules.
  Strict,

  /// The executor itself has error handling, so it can improve
  /// performance, but it can loose some error messages.
  Fast,
}

pub(crate) fn check_rules(
  registry: &Registry,
  doc: &ExecutableDocument,
  variables: Option<&Variables>,
  mode: ValidationMode,
  limit_depth: Option<usize>,
) -> Result<ValidationResult, Vec<ServerError>> {
  let mut depth = 0;

  let errors = match mode {
    ValidationMode::Strict => {
      let mut ctx = VisitorContext::new(registry, doc, variables);
      let mut visitor = VisitorNil
        .with(rules::ArgumentsOfCorrectType::default())
        .with(rules::DefaultValuesOfCorrectType)
        .with(rules::FieldsOnCorrectType)
        .with(rules::FragmentOnCompositeTypes)
        .with(rules::KnownArgumentNames::default())
        .with(rules::NoFragmentCycles::default())
        .with(rules::KnownFragmentNames::default())
        .with(rules::KnownTypeNames)
        .with(rules::NoUndefinedVariables::default())
        .with(rules::NoUnusedFragments::default())
        .with(rules::NoUnusedVariables::default())
        .with(rules::UniqueArgumentNames::default())
        .with(rules::UniqueVariableNames::default())
        .with(rules::VariablesAreInputTypes)
        .with(rules::VariablesInAllowedPosition::default())
        .with(rules::ScalarLeafs)
        .with(rules::PossibleFragmentSpreads::default())
        .with(rules::ProvidedNonNullArguments)
        .with(rules::KnownDirectives::default())
        .with(rules::DirectivesUnique)
        .with(rules::OverlappingFieldsCanBeMerged);
      visit(&mut visitor, &mut ctx, doc);

      let mut visitor = VisitorNil.with(visitors::DepthCalculate::new(&mut depth));
      visit(&mut visitor, &mut ctx, doc);
      ctx.errors
    }
    ValidationMode::Fast => {
      let mut ctx = VisitorContext::new(registry, doc, variables);
      let mut visitor = VisitorNil
        .with(rules::NoFragmentCycles::default())
        .with(visitors::DepthCalculate::new(&mut depth));
      visit(&mut visitor, &mut ctx, doc);
      ctx.errors
    }
  };

  if let Some(limit_depth) = limit_depth {
    if depth > limit_depth {
      return Err(vec![ServerError::new("Query is nested too deep", None)]);
    }
  }

  if !errors.is_empty() {
    return Err(errors.into_iter().map(Into::into).collect());
  }

  Ok(ValidationResult { depth })
}
