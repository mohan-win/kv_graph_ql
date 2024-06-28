//! GraphQL types.
//!
//! As per [GraphQL spec] (https://spec.graphql.org/October2021/#sec-Document), there are two kinds of document.
//! 1. TypeSystemDefinitionOrExtensionDocument (a.k.a) ServiceDocument
//! 2. ExecutableDocument.
//!
//! Since this crate only focuses on Generating OpenCRUD GraphQL TypeDefinitions for the given models in SDML file, only
//! ServiceDocument related types are definied.

mod service_document;

pub use graphql_value::*;
pub use service_document::*;
