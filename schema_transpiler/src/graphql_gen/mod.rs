//! GraphQL code generation.
//!
//! This module exposes necessary functions to generate GraphQL types for
//! SDML models.
//!
mod aux_type;
mod crud_api;
mod enum_type;
mod error;
mod input_type;
mod misc_type;
mod open_crud_name;
mod root_mutation_type;
mod root_query_type;
mod r#type;

use super::*;
use graphql_ast::*;
pub(crate) use open_crud_name::*;

/**
 * Public API
 */
pub use error::ErrorGraphQLGen;
pub type GraphQLGenResult<T> = Result<T, ErrorGraphQLGen>;
pub(crate) use crud_api::crud_api_def;
