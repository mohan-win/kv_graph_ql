/// Errors during SDML to GraphQL transpilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorGraphQLGen {
    /// Error in SDML file
    SDMLError {
        /// error details.
        error: &'static str,
        /// position in SDML file.
        pos: sdml_parser::ast::Span,
    },
}
