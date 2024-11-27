use sdml_parser::ast::Span;

/// Errors during SDML to GraphQL transpilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorGraphQLGen {
  /// Error in SDML file
  SDMLError {
    /// error details.
    error: String,
    /// position in SDML file.
    pos: sdml_parser::ast::Span,
  },
}

impl ErrorGraphQLGen {
  pub fn new_sdml_error((error, pos): (&'static str, Span)) -> Self {
    Self::SDMLError {
      error: error.to_string(),
      pos,
    }
  }
}
