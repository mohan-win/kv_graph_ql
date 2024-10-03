//! OpenCRUD type, field and field arg names.
mod fields;
mod types;

use super::*;
// Predefined GraphQL fields & types
pub(crate) const FIELD_TYPE_NAME_STRING: &str = "String";
pub(crate) const FIELD_TYPE_NAME_INT: &str = "Int";
pub(crate) const FIELD_TYPE_NAME_BOOL: &str = "Boolean";
pub(crate) const FIELD_TYPE_NAME_FLOAT: &str = "Float";
pub(crate) const FIELD_TYPE_SCALAR_DATETIME: &str = "DateTime";
// Field args
pub(crate) const FIELD_ARG_WHERE: &str = "where";
pub(crate) const FIELD_ARG_ORDER_BY: &str = "orderBy";
pub(crate) const FIELD_ARG_SKIP: &str = "skip";
pub(crate) const FIELD_ARG_AFTER: &str = "after";
pub(crate) const FIELD_ARG_BEFORE: &str = "before";
pub(crate) const FIELD_ARG_FIRST: &str = "first";
pub(crate) const FIELD_ARG_LAST: &str = "last";

pub(crate) use fields::*;
pub(crate) use types::*;
