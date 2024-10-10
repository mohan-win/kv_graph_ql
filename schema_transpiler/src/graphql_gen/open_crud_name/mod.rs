//! OpenCRUD type, field and field arg names.
use super::*;

pub(crate) mod fields;
pub(crate) use fields::FieldNamed;
pub(crate) mod types;
pub(crate) use types::Named;

// Predefined GraphQL fields & types
pub(crate) const FIELD_TYPE_NAME_STRING: &str = "String";
pub(crate) const FIELD_TYPE_NAME_INT: &str = "Int";
pub(crate) const FIELD_TYPE_NAME_BOOL: &str = "Boolean";
pub(crate) const FIELD_TYPE_NAME_FLOAT: &str = "Float";
pub(crate) const FIELD_TYPE_SCALAR_DATETIME: &str = "DateTime";
