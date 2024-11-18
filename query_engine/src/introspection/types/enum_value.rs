use crate::registry;

/// One possible value for a given Enum. Enum values are unique values, not a
/// placeholder for a string or numeric. However a Enum is returned in a JSON response as string.
pub struct __EnumValue<'a> {
    pub value: &'a registry::MetaEnumValue,
}
