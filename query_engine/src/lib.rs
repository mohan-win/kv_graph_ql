mod error;
mod execution;
mod introspection;
mod registry;
mod schema;
mod validation;

use graphql_parser;
use serde::{Deserialize, Serialize};
use serde_json::Result;

pub use graphql_value::ConstValue as Value;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[derive(Serialize, Deserialize)]
    struct GraphQLQuery {
        query: String,
        variables: String,
    }

    #[test]
    fn test_executable_doc_ast() {
        let graphql_query = GraphQLQuery {
            query: fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/test_data/crud_queries/create_user_with_posts.graphql"
            ))
            .unwrap(),
            variables: fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/test_data/crud_queries/create_user_with_posts.json"
            ))
            .unwrap(),
        };
        let graphql_schema = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/test_crud_api_def.graphql"
        ))
        .unwrap();
        // ToDo:: Validate query against schema.
        // ToDo:: Validate variable values against schema.
        let graphql_schema_ast = graphql_parser::parse_schema(graphql_schema).unwrap();
        let executable_doc_ast = graphql_parser::parse_query(graphql_query.query);
        eprintln!("{:#?}", executable_doc_ast);
        assert!(false, "testing!!");
    }
}
