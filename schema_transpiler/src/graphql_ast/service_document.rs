use super::*;
use std::fmt::{self, write, Display, Formatter, Write};

mod display_helpers;

/// A GraphQL file or request string defining a GraphQL service.
///
/// [Reference](https://spec.graphql.org/October2021/#Document).
#[derive(Debug, Clone)]
pub struct ServiceDocument {
    pub definitions: Vec<TypeSystemDefinition>,
}

/// A definition concerning the type system of a GraphQL service.
///
/// [Reference](https://spec.graphql.org/October2021/#TypeSystemDefinition). This enum also covers
/// [extensions](https://spec.graphql.org/October2021/#TypeSystemExtension).
#[derive(Debug, Clone)]
pub enum TypeSystemDefinition {
    /// The definition of the schema of the service.
    Schema(SchemaDefinition),
    /// The definition of a type in the service.
    Type(TypeDefinition),
    /// The definition of a directive in the service.
    Directive(DirectiveDefinition),
}

/// The definition of the schema in a GraphQL service.
///
/// [Reference](https://spec.graphql.org/October2021/#SchemaDefinition). This also covers
/// [extensions](https://spec.graphql.org/October2021/#SchemaExtension).
#[derive(Debug, Clone)]
pub struct SchemaDefinition {
    /// Whether the schema is an extension of another schema.
    pub extend: bool,
    /// The directives of the schema definition
    pub directives: Vec<ConstDirective>,
    /// The query root. This is always `Some` when the schema is not extended.
    pub query: Option<Name>,
    /// The mutation root, if present.
    pub mutation: Option<Name>,
    /// The subscription root, if present.
    pub subscription: Option<Name>,
}

/// The definition of a type in a GraphQL service.
///
/// [Reference](https://spec.graphql.org/October2021/#TypeDefinition). This also covers
/// [extensions](https://spec.graphql.org/October2021/#TypeExtension).
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    /// Whether the type is an extension of another type.
    pub extend: bool,
    /// The description of the type, if present. This is never present on an
    /// extension type.
    pub description: Option<String>,
    /// The name of the type
    pub name: Name,
    /// The directives of the type definition.
    pub directives: Vec<ConstDirective>,
    /// Which kind of type being defined, scalar, object, enum etc..
    pub kind: TypeKind,
}

/// A kind of type; scalar, object, enum, etc.
#[derive(Debug, Clone)]
pub enum TypeKind {
    /// A scalar type.
    Scalar,
    /// An object type.
    Object(ObjectType),
    /// An interface type.
    Interface(InterfaceType),
    /// A union type.
    Union(UnionType),
    /// An enum type
    Enum(EnumType),
    /// An input object type.
    InputObject(InputObjectType),
}

///
/// The definition of an object type.
///
/// [Reference](https://spec.graphql.org/October2021/#ObjectTypeDefinition).
#[derive(Debug, Clone)]
pub struct ObjectType {
    /// The interfaces implemented by object.
    pub implements: Vec<Name>,
    /// Fields of the object type.
    pub fields: Vec<FieldDefinition>,
}

/// The definition of a field inside an object or interface.
///
/// [Reference](https://spec.graphql.org/October2021/#FieldDefinition).
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    /// The description of the field.
    pub description: Option<String>,
    /// Name of the field.
    pub name: String,
    /// Arguments of the field.
    pub argments: Vec<InputValueDefinition>,
    /// Type of the field.
    pub ty: Type,
    /// Directive of the field.
    pub directives: Vec<ConstDirective>,
}

/// The definition of an interface type.
///
/// [Reference](https://spec.graphql.org/October2021/#InterfaceType).
#[derive(Debug, Clone)]
pub struct InterfaceType {
    /// Interfaces implemented by this interface.
    pub implements: Vec<Name>,
    /// Fields of the interface type.
    pub fields: Vec<FieldDefinition>,
}

/// The definition of a union type.
///
/// [Reference](https://spec.graphql.org/October2021/#UnionTypeDefinition).
#[derive(Debug, Clone)]
pub struct UnionType {
    /// The member types of the union.
    pub members: Vec<Name>,
}

/// The definition of an enum.
///
/// [Reference](https://spec.graphql.org/October2021/#EnumTypeDefinition).
#[derive(Debug, Clone)]
pub struct EnumType {
    /// Possible values of the enum.
    pub values: Vec<EnumValueDefinition>,
}

/// The definition of the value inside an enum.
///
/// [Reference](https://spec.graphql.org/October2021/#EnumValueDefinition).
#[derive(Debug, Clone)]
pub struct EnumValueDefinition {
    /// The description.
    pub description: Option<String>,
    /// The enum value name.
    pub value: Name,
    /// The directives of the enum value.
    pub directives: Vec<ConstDirective>,
}

/// The definition of an input object.
///
/// [Reference](https://spec.graphql.org/October2021/#InputObjectTypeDefinition).
#[derive(Debug, Clone)]
pub struct InputObjectType {
    /// The fields of the input object.
    pub fields: Vec<InputValueDefinition>,
}

/// The definition of an input value inside the arguments of a field.
///
/// [Reference](https://spec.graphql.org/October2021/#InputValueDefinition).
#[derive(Debug, Clone)]
pub struct InputValueDefinition {
    /// Description of the argument.
    pub description: Option<String>,
    /// Name of the arument.
    pub name: Name,
    /// Type of the argument
    pub ty: Type,
    /// The default value of the argument, if any.
    pub default_value: Option<ConstValue>,
    /// The directives of the input value.
    pub directives: Vec<ConstDirective>,
}

impl fmt::Display for InputValueDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\n")?; // Note: Start each input argument with a newline so that they needn't be comma separated.
        display_helpers::write_description_ln(f, &self.description)?;
        write!(f, "{}: {}", self.name, self.ty)?;
        if self.default_value.is_some() {
            write!(f, " = {}", self.default_value.as_ref().unwrap())?;
        }
        self.directives
            .iter()
            .try_for_each(|directive| write!(f, " {}", directive))
    }
}

/// The definition of a directive in a service
///
/// [Reference](https://spec.graphql.org/October2021/#DirectiveDefinition).
#[derive(Debug, Clone)]
pub struct DirectiveDefinition {
    /// The description of the directive.
    pub description: Option<String>,
    /// Name of the directive.
    pub name: Name,
    /// Arguments to the directive.
    pub arguments: Vec<InputValueDefinition>,
    /// Whether the directive can be repeated.
    pub is_repeatable: bool,
    /// Locations where the directive applies to.
    pub locations: Vec<DirectiveLocation>,
}

/// Where the directive can be applied to.
///
/// [Reference](https://spec.graphql.org/October2021/#DirectiveLocation).
#[derive(Debug, Clone)]
pub enum DirectiveLocation {
    // ExecutableDirectiveLocation
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    VariableDefinition,

    // TypeSystemDirectiveLocation
    /// A [schema](struct.SchemaDefinition.html)
    Schema,
    /// A [scalar](struct.TypeKind.html#variant.Scalar)
    Scalar,
    /// An [object](struct.ObjectType.html)
    Object,
    /// A [field definition](struct.FieldDefinition.html)
    FieldDefinition,
    /// An [input value definition](struct.InputValueDefinition.html) as the
    /// argument of the field but not on an input object.
    ArgumentDefinition,
    /// An [interface](struct.InterfaceType.html).
    Interface,
    /// A [union](struct.UnionType.html).
    Union,
    /// An [enum](struct.EnumType.html).
    Enum,
    /// An [value of an enum](struct.EnumValueDefinition.html).
    EnumValue,
    /// An [input object](struct.InputObjectType.html).
    InputObject,
    /// An [input value definition](struct.InputValueDefinition.html) on an
    /// input object but not on a argument.
    InputFieldDefinition,
}

/// A GraphQL type, for example `String` or `[String!]!`.
///
/// [Reference](https://spec.graphql.org/October2021/#Type).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Type {
    /// The base type.
    pub base: BaseType,
    /// Whether the type is nullable.
    pub nullable: bool,
}

impl Type {
    /// Create a type from the type string.
    #[must_use]
    pub fn new(ty: &str) -> Option<Self> {
        let (nullable, ty) = if let Some(rest) = ty.strip_suffix('!') {
            (false, rest)
        } else {
            (true, ty)
        };

        Some(Self {
            base: if let Some(ty) = ty.strip_prefix('[') {
                BaseType::List(Box::new(Self::new(ty.strip_suffix(']')?)?))
            } else {
                BaseType::Named(Name::new(ty))
            },
            nullable,
        })
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.base.fmt(f)?;
        if !self.nullable {
            f.write_char('!')?;
        }
        Ok(())
    }
}

/// A GraphQL base type, for example `String` or `[String!]`. This does not
/// include whether the type is nullable; for that see [Type](struct.Type.html).
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BaseType {
    /// A named type, such as `String`.
    Named(Name),
    /// A list type, such as `[String]`.
    List(Box<Type>),
}

impl Display for BaseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(name) => f.write_str(name),
            Self::List(ty) => write!(f, "[{}]", ty),
        }
    }
}

/// A const GraphQL directive, such as `@deprecated(reason: "Use the other
/// field)`. This differs from [`Directive`](struct.Directive.html) in that it
/// uses [`ConstValue`](enum.ConstValue.html) instead of
/// [`Value`](enum.Value.html).
///
/// [Reference](https://spec.graphql.org/October2021/#Directive).
#[derive(Debug, Clone)]
pub struct ConstDirective {
    /// The name of the directive.
    pub name: Name,
    /// The arguments to the directive.
    pub arguments: Vec<(Name, ConstValue)>,
}

impl ConstDirective {
    /// Convert this `ConstDirective` into a `Directive`.
    #[must_use]
    pub fn into_directive(self) -> Directive {
        Directive {
            name: self.name,
            arguments: self
                .arguments
                .into_iter()
                .map(|(name, value)| (name, value.into_value()))
                .collect(),
        }
    }

    /// Get the argument with the given name.
    #[must_use]
    pub fn get_argument(&self, name: &str) -> Option<&ConstValue> {
        self.arguments
            .iter()
            .find(|item| item.0 == name)
            .map(|item| &item.1)
    }
}

impl fmt::Display for ConstDirective {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let args: String = self
            .arguments
            .iter()
            .map(|(name, const_value)| format!("{name}: {const_value}, "))
            .collect();
        let args = args.trim_end_matches([',', ' ']);
        write!(f, "@{}({})", self.name, args)
    }
}

/// A GraphQL directive, such as `@deprecated(reason: "Use the other field")`.
///
/// [Reference](https://spec.graphql.org/October2021/#Directive).
#[derive(Debug, Clone)]
pub struct Directive {
    /// The name of the directive.
    pub name: Name,
    /// The arguments to the directive.
    pub arguments: Vec<(Name, Value)>,
}

impl Directive {
    /// Attempt to convert this `Directive` into a `ConstDirective`.
    #[must_use]
    pub fn into_const(self) -> Option<ConstDirective> {
        Some(ConstDirective {
            name: self.name,
            arguments: self
                .arguments
                .into_iter()
                .map(|(name, value)| Some((name, value.into_const()?)))
                .collect::<Option<_>>()?,
        })
    }

    /// Get the argument with the given name.
    #[must_use]
    pub fn get_argument(&self, name: &str) -> Option<&Value> {
        self.arguments
            .iter()
            .find(|item| item.0 == name)
            .map(|item| &item.1)
    }
}

#[cfg(test)]
mod test {
    use graphql_value::{ConstValue, Name};

    use super::{ConstDirective, InputValueDefinition, Type};

    #[test]
    fn test_const_directive_display_trait() {
        let deprecated_directive = ConstDirective {
            name: Name::new("deprecated"),
            arguments: vec![(
                Name::new("reason"),
                ConstValue::String("Use some_other_field".to_string()),
            )],
        };

        assert_eq!(
            "@deprecated(reason: \"Use some_other_field\")",
            deprecated_directive.to_string()
        );

        let some_directive = ConstDirective {
            name: Name::new("some"),
            arguments: vec![
                (
                    Name::new("arg1"),
                    ConstValue::String("String value".to_string()),
                ),
                (Name::new("arg2"), ConstValue::Boolean(true)),
                (
                    Name::new("arg3"),
                    ConstValue::String("Another string value".to_string()),
                ),
            ],
        };

        assert_eq!(
            "@some(arg1: \"String value\", arg2: true, arg3: \"Another string value\")",
            some_directive.to_string()
        );
    }

    #[test]
    fn test_input_value_definition_display_trait() {
        let deprecated_directive = ConstDirective {
            name: Name::new("deprecated"),
            arguments: vec![(
                Name::new("reason"),
                ConstValue::String("Use some_other_field".to_string()),
            )],
        };

        let some_directive = ConstDirective {
            name: Name::new("some"),
            arguments: vec![
                (
                    Name::new("arg1"),
                    ConstValue::String("String value".to_string()),
                ),
                (Name::new("arg2"), ConstValue::Boolean(true)),
                (
                    Name::new("arg3"),
                    ConstValue::String("Another string value".to_string()),
                ),
            ],
        };

        let id_input_value = InputValueDefinition {
            description: Some("This is some id with default value as def_id".to_string()),
            name: Name::new("id"),
            ty: Type::new("ID").unwrap(),
            default_value: Some(ConstValue::String("def_id".to_string())),
            directives: vec![some_directive, deprecated_directive],
        };

        let id_input_value_expected_str = r#"
"""This is some id with default value as def_id"""
id: ID = "def_id" @some(arg1: "String value", arg2: true, arg3: "Another string value") @deprecated(reason: "Use some_other_field")"#;
        assert_eq!(id_input_value_expected_str, id_input_value.to_string());

        let id_input_value = InputValueDefinition {
            description: Some("This is some id with default value as def_id".to_string()),
            name: Name::new("id"),
            ty: Type::new("ID").unwrap(),
            default_value: None,
            directives: vec![],
        };

        let id_input_value_expected_str = r#"
"""This is some id with default value as def_id"""
id: ID"#;
        assert_eq!(id_input_value_expected_str, id_input_value.to_string());

        let id_input_value = InputValueDefinition {
            description: None,
            name: Name::new("id"),
            ty: Type::new("ID").unwrap(),
            default_value: None,
            directives: vec![],
        };

        let id_input_value_expected_str = r#"
id: ID"#;
        assert_eq!(id_input_value_expected_str, id_input_value.to_string());
    }
}
