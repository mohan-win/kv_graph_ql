mod rules;
mod utlis;
mod visitor;

use graphql_parser::types::ExecutableDocument;

use crate::{error::ServerError, registry::Registry, Variables};

#[derive(Debug, Clone, Copy)]
pub struct ValidationResult {
    /// Query complexity.
    pub complexity: usize,

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
    limit_complexity: Option<usize>,
    limit_depth: Option<usize>,
) -> Result<ValidationResult, Vec<ServerError>> {
    unimplemented!()
}
