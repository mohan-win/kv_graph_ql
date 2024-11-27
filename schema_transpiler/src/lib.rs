mod graphql_ast;
mod graphql_gen;

use sdml_parser::ast::{self as sdml_ast, DataModel};

/**
 * Public API
 */
pub use graphql_gen::ErrorGraphQLGen;

/// Generates [OpenCRUD][https://www.opencrud.org/] API definitions for the given data model.
/// ### Arguments
/// * SDML AST of the data model in the SDML file.
/// ### Returns
/// * GraphQL schema document with all the necessary type definitions as per
/// OpenCRUD spec.
pub fn generate_crud_api<'src>(
  data_model: &DataModel<'src>,
) -> Result<String, ErrorGraphQLGen> {
  let crud_api = graphql_gen::crud_api_def(data_model)?;
  Ok(crud_api.iter().fold(String::new(), |mut acc, graphql_ty| {
    acc.push_str(&graphql_ty.to_string());
    acc
  }))
}

#[cfg(test)]
mod tests {
  use super::generate_crud_api;
  use chumsky::prelude::*;
  use sdml_parser::parser;
  use std::fs;

  #[test]
  fn test_generate_crud_api_def() {
    let sdml_str = fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/test_generate_crud_api_def.sdml"
    ))
    .unwrap();
    let mut expected_graphql_str = fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/test_generate_crud_api_def.graphql"
    ))
    .unwrap();
    expected_graphql_str.retain(|c| !c.is_whitespace());

    let sdml_decls = parser::delcarations()
      .parse(&sdml_str)
      .into_result()
      .unwrap();
    let sdml_ast = parser::semantic_analysis(sdml_decls).unwrap();
    let mut actual_crud_api_graphql_str = generate_crud_api(&sdml_ast).unwrap();
    actual_crud_api_graphql_str.retain(|c| !c.is_whitespace());
    assert_eq!(expected_graphql_str, actual_crud_api_graphql_str)
  }
}
