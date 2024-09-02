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
    pub fn str(&self) -> Option<&'src str> {
        if let Token::Str(str, _) = self {
            Some(str.trim_matches('"'))
        } else {
            None
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
    pub fn try_get_ident_name(&self) -> Result<&'src str, (&'static str, Span)> {
        if let Token::Ident(name, _) = self {
            Ok(name)
        } else {
            Err((
                "Can't get the identifier name for a non-identifier",
                self.span(),
            ))
        }
    }
    pub fn try_get_graphql_name(&self) -> Result<graphql_value::Name, (&'static str, Span)> {
        match self {
            Token::Ident(name, _) => Ok(graphql_value::Name::new(name)),
            other => Err(("GraphQL name should be a valid identifier", other.span())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataModel<'src> {
    /// Map of config name to its declarations.
    configs: HashMap<&'src str, ConfigDecl<'src>>,
    /// Map of enum name to its declarations.
    enums: HashMap<&'src str, EnumDecl<'src>>,
    /// Map of model name to its declarations.
    models: HashMap<&'src str, ModelDecl<'src>>,
    /// Map of valid relations with fully formed edges.
    /// Available only after semantic_analysis phase.
    relations: HashMap<&'src str, (RelationEdge<'src>, Option<RelationEdge<'src>>)>,
}

impl<'src> DataModel<'src> {
    pub fn new() -> DataModel<'src> {
        DataModel {
            configs: HashMap::new(),
            enums: HashMap::new(),
            models: HashMap::new(),
            relations: HashMap::new(),
        }
    }
    pub fn configs(&self) -> &HashMap<&'src str, ConfigDecl<'src>> {
        &self.configs
    }
    pub fn enums(&self) -> &HashMap<&'src str, EnumDecl<'src>> {
        &self.enums
    }
    pub fn models(&self) -> &HashMap<&'src str, ModelDecl<'src>> {
        &self.models
    }
    pub fn relations(
        &self,
    ) -> &HashMap<&'src str, (RelationEdge<'src>, Option<RelationEdge<'src>>)> {
        &self.relations
    }
    pub(crate) fn configs_mut(&mut self) -> &mut HashMap<&'src str, ConfigDecl<'src>> {
        &mut self.configs
    }
    pub(crate) fn enums_mut(&mut self) -> &mut HashMap<&'src str, EnumDecl<'src>> {
        &mut self.enums
    }
    pub(crate) fn models_mut(&mut self) -> &mut HashMap<&'src str, ModelDecl<'src>> {
        &mut self.models
    }
    pub(crate) fn relations_mut(
        &mut self,
    ) -> &mut HashMap<&'src str, (RelationEdge<'src>, Option<RelationEdge<'src>>)> {
        &mut self.relations
    }
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

/// Represents an entity inside the application domain.
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
    Primitive {
        r#type: PrimitiveType,
        token: Token<'src>,
    },
    Enum(Token<'src>),
    /// If field type is other model type, then its a `Relation`.
    Relation(RelationEdge<'src>),
    /// If the field type is Enum or Relation, in the first pass it will be set to Unknown with identifier token.
    /// Then only during scemantic analysis its actual user defined type is determined.
    Unknown(Token<'src>),
}

impl<'src> Type<'src> {
    pub fn token(&self) -> &Token<'src> {
        match self {
            Self::Primitive { token, .. } => token,
            Self::Enum(token) => token,
            Self::Relation(relation_edge) => relation_edge.referenced_model_name(),
            Self::Unknown(token) => token,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationEdge<'src> {
    /// One-side of the relation, for both
    /// Left side of the 1-to-1 relation and
    /// One side of the 1-to-many relation.
    OneSideRelation {
        relation_name: Token<'src>,
        referenced_model_name: Token<'src>,
    },
    /// One-side of the relation on the right in 1-to-1 relation.
    OneSideRelationRight {
        relation_name: Token<'src>,
        /// Name of the relational scalar field storing the foreign key values.
        /// It should be marked with @unique attribute, to make (1-to-1) explicit in the schema.
        scalar_field_name: Token<'src>,
        referenced_model_name: Token<'src>,
        /// Name of the field (should be either @id or @unique) in the referenced model.
        referenced_field_name: Token<'src>,
    },
    /// Many-side of the relation, capturing the required information for
    /// a. Many side of 1-to-many relation,
    /// b. Both sides fo the many-to-many relation.
    /// c. Many side of the self-to-many relation,
    ManySideRelation {
        relation_name: Token<'src>,
        /// Name of the relational scalar field storing the foreign key values.
        scalar_field_name: Token<'src>,
        referenced_model_name: Token<'src>,
        /// Name of the field (should be either @id or @unique) in the referenced model.
        referenced_field_name: Token<'src>,
    },
    /// Self relation of type 1-to-1
    SelfOneToOneRelation {
        relation_name: Token<'src>,
        /// Name of the scalar field name. It should be marked with @unique attribute, to make (1-to-1) explicit in the schema.
        scalar_field_name: Token<'src>,
        referenced_model_name: Token<'src>,
        /// Name of the referened field (should be either @id or @unique) in the model.
        referenced_field_name: Token<'src>,
    },
}

impl<'src> RelationEdge<'src> {
    pub fn relation_name(&self) -> &Token<'src> {
        match self {
            Self::OneSideRelation { relation_name, .. } => relation_name,
            Self::OneSideRelationRight { relation_name, .. } => relation_name,
            Self::ManySideRelation { relation_name, .. } => relation_name,
            Self::SelfOneToOneRelation { relation_name, .. } => relation_name,
        }
    }

    fn referenced_model_name(&self) -> &Token<'src> {
        match self {
            Self::OneSideRelation {
                referenced_model_name,
                ..
            } => referenced_model_name,
            Self::OneSideRelationRight {
                referenced_model_name,
                ..
            } => referenced_model_name,
            Self::ManySideRelation {
                referenced_model_name,
                ..
            } => referenced_model_name,
            Self::SelfOneToOneRelation {
                referenced_model_name,
                ..
            } => referenced_model_name,
        }
    }
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
pub enum AttribArg<'src> {
    Args(Vec<NamedArg<'src>>),
    Function(Token<'src>),
    Ident(Token<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamedArg<'src> {
    pub arg_name: Token<'src>,
    pub arg_value: Token<'src>,
}
