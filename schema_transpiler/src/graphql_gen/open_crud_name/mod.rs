//! OpenCRUD type, field and field arg names.
use super::*;

pub mod fields;
pub use fields::FieldNamed;
pub mod types;
pub use types::Named;

// Predefined GraphQL fields & types
pub const FIELD_TYPE_NAME_STRING: &str = "String";
pub const FIELD_TYPE_NAME_INT: &str = "Int";
pub const FIELD_TYPE_NAME_BOOL: &str = "Boolean";
pub const FIELD_TYPE_NAME_FLOAT: &str = "Float";
pub const FIELD_TYPE_SCALAR_DATETIME: &str = "DateTime";
