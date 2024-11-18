#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum __TypeKind {
    /// Indicates this type is a scalar.
    Scalar,
    /// Indicates this type is an object. `fields` and `interfaces` are valid
    /// fields.
    Object,
    /// Indicates this type is an interface. `fields` and `possible_types` are valid fields.
    Interface,
    /// Indicates this type is an union. `possible_types` is a valid field.
    Union,
    /// Indicates this type is an enum. `enum_values` is a valid field
    Enum,
    /// Indicates this type is an input object. `input_fields` is a valid field.
    InputObject,
    /// Indicates this type is a list. `of_type` is a valid field.
    List,
    /// Indicates this type is a non-null. `of_type` is a valid field.
    NonNull,
}
