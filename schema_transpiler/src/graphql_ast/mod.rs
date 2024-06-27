//! GraphQL AST types for GraphQL Schema.
//!
//! This follows the [June 2018 edition of the GraphQL spec](https://spec.graphql.org/October2021/).

use graphql_value::{ConstValue, Name, Value};
use std::fmt::{self, Display, Formatter, Write};

/// Module which defines the GraphQL Schema Definition Language types.
mod sdl_ast;

/// The type of an operation; `query`, `mutation` or `subscription`.
///
/// [Reference](https://spec.graphql.org/October2021/#OperationType).
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum OperationType {
    /// A query.
    Query,
    /// A mutation.
    Mutation,
    /// A subscription.
    Subscription,
}

impl Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Query => "query",
            Self::Mutation => "mutation",
            Self::Subscription => "subscription",
        })
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
