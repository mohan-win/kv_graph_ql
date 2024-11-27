#![allow(dead_code)]

//! The ServiceDocument types defined here are similar to types defined in
//! graphql_parser crate except that,
//! 1. These types exclude Position information, as they are not relevant for graphql_gen module.
//! 2. Includes display_helpers, to convert these ServiceDocument::* types into GraphQL SDL.
use super::*;
pub mod display_helpers;

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
  /// The description of the type, if present. This is never present on an
  /// extension type.
  pub description: Option<String>,
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
#[allow(dead_code)]
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
  pub name: Name,
  /// Arguments of the field.
  pub arguments: Vec<InputValueDefinition>,
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

#[derive(Debug, Clone, PartialEq)]
pub enum TypeMod {
  /// Non-optional type modifier. Type!
  NonOptional,
  /// Optional type modifier. Type
  Optional,
  /// Array type modifier. \[ElementType!\]!
  Array,
  /// Optional Array with, Non-optional elements. \[ElementType!\]
  ArrayOptional,
}

impl From<sdml_parser::ast::FieldTypeMod> for TypeMod {
  fn from(value: sdml_parser::ast::FieldTypeMod) -> Self {
    match value {
      sdml_parser::ast::FieldTypeMod::NonOptional => TypeMod::NonOptional,
      sdml_parser::ast::FieldTypeMod::Optional => TypeMod::Optional,
      sdml_parser::ast::FieldTypeMod::Array => TypeMod::Array,
    }
  }
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
  pub fn new(ty: &str, type_mod: TypeMod) -> Type {
    let ty = match type_mod {
      TypeMod::NonOptional => format!("{ty}!"),
      TypeMod::Optional => ty.to_string(),
      TypeMod::Array => format!("[{ty}!]!"),
      TypeMod::ArrayOptional => format!("[{ty}!]"),
    };

    Type::new_from_str(&ty).expect("Pass a valid type name!")
  }

  /// Create a type from the type string.
  #[must_use]
  pub fn new_from_str(ty: &str) -> Option<Self> {
    let (nullable, ty) = if let Some(rest) = ty.strip_suffix('!') {
      (false, rest)
    } else {
      (true, ty)
    };

    Some(Self {
      base: if let Some(ty) = ty.strip_prefix('[') {
        BaseType::List(Box::new(Self::new_from_str(ty.strip_suffix(']')?)?))
      } else {
        BaseType::Named(Name::new(ty))
      },
      nullable,
    })
  }

  /// Returns the graphql type name for the given SDML primitve type.
  pub fn map_sdml_type_to_graphql_ty_name(
    r#type: &sdml_parser::ast::PrimitiveType,
  ) -> String {
    use crate::graphql_gen::{
      FIELD_TYPE_NAME_BOOL, FIELD_TYPE_NAME_INT, FIELD_TYPE_NAME_STRING,
      FIELD_TYPE_SCALAR_DATETIME,
    };
    use sdml_parser::ast::PrimitiveType;
    match r#type {
      PrimitiveType::ShortStr | PrimitiveType::LongStr => FIELD_TYPE_NAME_STRING,
      PrimitiveType::DateTime => FIELD_TYPE_SCALAR_DATETIME,
      PrimitiveType::Boolean => FIELD_TYPE_NAME_BOOL,
      PrimitiveType::Int32 | PrimitiveType::Int64 => FIELD_TYPE_NAME_INT,
      PrimitiveType::Float64 => FIELD_TYPE_NAME_BOOL,
    }
    .to_string()
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
    self
      .arguments
      .iter()
      .find(|item| item.0 == name)
      .map(|item| &item.1)
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
    self
      .arguments
      .iter()
      .find(|item| item.0 == name)
      .map(|item| &item.1)
  }
}
