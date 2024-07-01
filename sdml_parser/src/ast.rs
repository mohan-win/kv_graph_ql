//! Abstract Synctax Tree (AST) types of Simple Data Modeling Language (SDML).

use std::{cell::RefCell, collections::HashMap};

use chumsky::span::SimpleSpan;

pub type Span = SimpleSpan<usize>;

#[derive(Debug, Clone)]
pub enum Token<'src> {
    // Litrals
    Ident(&'src str, Span),
    Str(&'src str, Span),
    Int(i64, Span),
    Float(f64, Span),
    Bool(bool, Span),
}
impl<'src> PartialEq for Token<'src> {
    // Important Note: Implementing Partial equal for Token, which only compares the
    // actual token and doesn't compare the location (span) in which it is found.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Token::Ident(self_ident, _), Token::Ident(other_ident, _)) => {
                self_ident == other_ident
            }
            (Token::Str(self_str, _), Token::Str(other_str, _)) => self_str == other_str,
            (Token::Int(self_int, _), Token::Int(other_int, _)) => self_int == other_int,
            (Token::Float(self_float, _), Token::Float(other_float, _)) => {
                self_float == other_float
            }
            (Token::Bool(self_bool, _), Token::Bool(other_bool, _)) => self_bool == other_bool,
            (_, _) => false,
        }
    }
}

impl<'src> Token<'src> {
    /// If the token is an identifier, returns its name
    /// if not, it returns None
    pub fn ident_name(&self) -> Option<&'src str> {
        if let Token::Ident(name, _) = self {
            Some(name)
        } else {
            None
        }
    }
    pub fn try_ident_name(&self) -> Result<&'src str, (&'static str, Span)> {
        if let Token::Ident(name, _) = self {
            Ok(name)
        } else {
            Err((
                "Can't get the identifier name for a non-identifier",
                self.span(),
            ))
        }
    }
    pub fn span(&self) -> Span {
        match self {
            Token::Ident(_, sp) => *sp,
            Token::Str(_, sp) => *sp,
            Token::Int(_, sp) => *sp,
            Token::Float(_, sp) => *sp,
            Token::Bool(_, sp) => *sp,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataModel<'src> {
    pub configs: HashMap<&'src str, ConfigDecl<'src>>,
    pub enums: HashMap<&'src str, EnumDecl<'src>>,
    pub models: HashMap<&'src str, ModelDecl<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration<'src> {
    Config(ConfigDecl<'src>),
    Enum(EnumDecl<'src>),
    Model(ModelDecl<'src>),
}

impl<'src> Declaration<'src> {
    pub fn name(&self) -> &'src Token {
        match self {
            Declaration::Config(c) => &c.name,
            Declaration::Enum(e) => &e.name,
            Declaration::Model(m) => &m.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigDecl<'src> {
    pub name: Token<'src>,
    pub config_pairs: Vec<ConfigPair<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigPair<'src> {
    pub name: Token<'src>,
    pub value: ConfigValue<'src>,
}

#[derive(Debug, Clone)]
pub enum ConfigValue<'src> {
    Str(&'src str, Span),
    Int(i64, Span),
    Float(f64, Span),
    Bool(bool, Span),
}

impl<'src> PartialEq for ConfigValue<'src> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ConfigValue::Str(self_str, _), ConfigValue::Str(other_str, _)) => {
                self_str == other_str
            }
            (ConfigValue::Int(self_int, _), ConfigValue::Int(other_int, _)) => {
                self_int == other_int
            }
            (ConfigValue::Float(self_float, _), ConfigValue::Float(other_float, _)) => {
                self_float == other_float
            }
            (ConfigValue::Bool(self_bool, _), ConfigValue::Bool(other_bool, _)) => {
                self_bool == other_bool
            }
            (_, _) => false,
        }
    }
}

impl<'src> TryFrom<Token<'src>> for ConfigValue<'src> {
    type Error = String;
    fn try_from(value: Token<'src>) -> Result<Self, Self::Error> {
        match value {
            Token::Bool(b, s) => Ok(ConfigValue::Bool(b, s)),
            Token::Float(f, s) => Ok(ConfigValue::Float(f, s)),
            Token::Int(i, s) => Ok(ConfigValue::Int(i, s)),
            Token::Str(str, s) => Ok(ConfigValue::Str(str, s)),
            t => Err(format!("Token {:?} can't turned into string", t)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDecl<'src> {
    pub name: Token<'src>,
    pub elements: Vec<Token<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelDecl<'src> {
    pub name: Token<'src>,
    pub fields: Vec<FieldDecl<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl<'src> {
    pub name: Token<'src>,
    pub field_type: FieldType<'src>,
    pub attributes: Vec<Attribute<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldType<'src> {
    r#type: RefCell<Type<'src>>, // Note: interier mutability, this is because the field_type for custom types set to Type::Unknown in the first pass. And then in the later pass actual type is determined.
    pub is_optional: bool,
    pub is_array: bool,
}

impl<'src> FieldType<'src> {
    pub fn new(r#type: Type<'src>, is_optional: bool, is_array: bool) -> FieldType<'src> {
        FieldType {
            r#type: RefCell::new(r#type),
            is_optional,
            is_array,
        }
    }
    pub fn r#type(&self) -> std::cell::Ref<Type<'src>> {
        self.r#type.borrow()
    }
    pub(crate) fn set_type(&self, new_type: Type<'src>) {
        *self.r#type.borrow_mut() = new_type;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type<'src> {
    Primitive(PrimitiveType),
    Enum(Token<'src>),
    /// If field type is other model type, then its a `Relation`.
    Relation(Token<'src>),
    /// If the field type is Enum or Relation, in the first pass it will be set to Unknown with identifier token.
    /// Then only during scemantic analysis its actual user defined type is determined.
    Unknown(Token<'src>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    ShortStr,
    LongStr,
    DateTime,
    Boolean,
    Int32,
    Int64,
    Float64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute<'src> {
    pub name: Token<'src>,
    pub arg: Option<AttribArg<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttribArg<'src> {
    pub name: Token<'src>,
    pub is_function: bool,
}
