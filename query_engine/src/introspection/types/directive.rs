use std::collections::HashSet;

#[allow(non_camel_case_types)]
pub enum __DirectiveLocation {
    /// Location adjacent to a query operation.
    QUERY,

    /// Location adjacent to a mutation operation.
    MUTATION,

    /// Location adjacent to a subscription operation.
    SUBSCRIPTION,

    /// Location adjacent to a field.
    FIELD,

    /// Location adjacent to a fragment definition.
    FRAGMENT_DEFINITION,

    /// Location adjacent to a fragment spread.
    FRAGMENT_SPREAD,

    /// Location adjacent to an inline fragment.
    INLINE_FRAGMENT,

    /// Location adjacent to a variable definition.
    VARIABLE_DEFINITION,

    /// Location adjacent to a schema definition.
    SCHEMA,

    /// Location adjacent to a scalar definition.
    SCALAR,

    /// Location adjacent to an object type definition.
    OBJECT,

    /// Location adjacent to a field definition.
    FIELD_DEFINITION,

    /// Location adjacent to an argument definition.
    ARGUMENT_DEFINITION,

    /// Location adjacent to an interface definition.
    INTERFACE,

    /// Location adjacent to a union definition.
    UNION,

    /// Location adjacent to an enum definition.
    ENUM,

    /// Location adjacent to an enum value definition.
    ENUM_VALUE,

    /// Location adjacent to an input object type definition.
    INPUT_OBJECT,

    /// Location adjacent to an input object field definition.
    INPUT_FIELD_DEFINITION,
}
