mod fields;
mod types;

use super::*;
// Predefined GraphQL fields & types
pub const FIELD_TYPE_NAME_STRING: &str = "String";
pub const FIELD_TYPE_NAME_INT: &str = "Int";
pub const FIELD_TYPE_NAME_BOOL: &str = "Boolean";
pub const FIELD_TYPE_NAME_FLOAT: &str = "Float";
pub const FIELD_TYPE_SCALAR_DATETIME: &str = "DateTime";
// Field args
pub const FIELD_ARG_WHERE: &str = "where";
pub const FIELD_ARG_ORDER_BY: &str = "orderBy";
pub const FIELD_ARG_SKIP: &str = "skip";
pub const FIELD_ARG_AFTER: &str = "after";
pub const FIELD_ARG_BEFORE: &str = "before";
pub const FIELD_ARG_FIRST: &str = "first";
pub const FIELD_ARG_LAST: &str = "last";

pub use fields::*;
pub use types::*;
