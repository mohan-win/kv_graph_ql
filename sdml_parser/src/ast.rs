//! Abstract Synctax Tree (AST) types of Simple Data Modeling Language (SDML).

use std::{cell::RefCell, collections::HashMap};

use crate::parser::semantic_analysis;
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
            (Token::Bool(self_bool, _), Token::Bool(other_bool, _)) => {
                self_bool == other_bool
            }
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
    pub fn try_get_graphql_name(
        &self,
    ) -> Result<graphql_value::Name, (&'static str, Span)> {
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
    /// Configs sorted by its name in chronogical order.
    pub fn configs_sorted(&self) -> Vec<&ConfigDecl<'src>> {
        let mut config_names = self.enums.keys().collect::<Vec<&&'src str>>();
        config_names.sort();
        config_names
            .into_iter()
            .rfold(Vec::new(), |mut acc, config_name| {
                acc.push(self.configs.get(config_name).unwrap());
                acc
            })
    }

    pub fn enums(&self) -> &HashMap<&'src str, EnumDecl<'src>> {
        &self.enums
    }
    /// Enums sorted by its name in chronogical order.
    pub fn enums_sorted(&self) -> Vec<&EnumDecl<'src>> {
        let mut enum_names = self.enums.keys().collect::<Vec<&&'src str>>();
        enum_names.sort();
        enum_names
            .into_iter()
            .rfold(Vec::new(), |mut acc, enum_name| {
                acc.push(self.enums.get(enum_name).unwrap());
                acc
            })
    }
    pub fn models(&self) -> &HashMap<&'src str, ModelDecl<'src>> {
        &self.models
    }
    /// Models sorted by its name in chronogical order.
    pub fn models_sorted(&self) -> Vec<&ModelDecl<'src>> {
        let mut model_names = self.models.keys().collect::<Vec<&&'src str>>();
        model_names.sort();
        model_names
            .into_iter()
            .rfold(Vec::new(), |mut acc, model_name| {
                acc.push(self.models.get(model_name).unwrap());
                acc
            })
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

#[derive(Debug, Clone)]
pub struct ModelFields<'src, 'b> {
    pub relation_fields: Vec<&'b FieldDecl<'src>>,
    pub relation_scalar_fields: Vec<&'b FieldDecl<'src>>,
    pub id_fields: Vec<(&'b FieldDecl<'src>, bool)>, // Vec<(field, is_auto_generated)>
    pub unique_fields: Vec<&'b FieldDecl<'src>>,
    pub non_unique_fields: Vec<&'b FieldDecl<'src>>,
}

impl<'src, 'b> ModelDecl<'src> {
    /// Get Model fields.
    pub fn get_fields(&'b self) -> ModelFields<'src, 'b> {
        self.get_fields_internal(false) // IMPORTNAT: Don't allow unknown_field_type when fields are accessed from outside the crate.
    }

    pub(crate) fn get_fields_internal(
        &'b self,
        allow_unknown_field_type: bool,
    ) -> ModelFields<'src, 'b> {
        let mut result = ModelFields {
            relation_fields: Vec::new(),
            relation_scalar_fields: Vec::new(),
            id_fields: Vec::new(),
            unique_fields: Vec::new(),
            non_unique_fields: Vec::new(),
        };

        let mut relation_scalar_field_names = Vec::new();
        self.fields
            .iter()
            .for_each(|field| match &*field.field_type.r#type() {
                Type::Unknown(..) => {
                    if !allow_unknown_field_type {
                        panic!("Can't allow unknown field type.")
                    }
                }
                Type::Relation(edge) => {
                    result.relation_fields.push(field);
                    edge.scalar_field_name().map(|fld_name| {
                        relation_scalar_field_names.push(fld_name.ident_name().unwrap())
                    });
                }
                Type::Primitive { .. } | Type::Enum { .. } => {
                    if field.is_auto_gen_id() {
                        result.id_fields.push((field, true));
                    } else if field.has_id_attrib() {
                        result.id_fields.push((field, false));
                    } else if field.has_unique_attrib() {
                        result.unique_fields.push(field);
                    } else {
                        result.non_unique_fields.push(field);
                    }
                }
            });

        let mut relation_scalar_fields = Vec::new();
        // Filter-out relation scalar fields from unique & non-unique fields.
        result.unique_fields = result
            .unique_fields
            .into_iter()
            .filter(|field| {
                if relation_scalar_field_names.contains(&field.name.ident_name().unwrap())
                {
                    relation_scalar_fields.push(*field);
                    false
                } else {
                    true
                }
            })
            .collect();
        result.non_unique_fields = result
            .non_unique_fields
            .into_iter()
            .filter(|field| {
                if relation_scalar_field_names.contains(&field.name.ident_name().unwrap())
                {
                    relation_scalar_fields.push(*field);
                    false
                } else {
                    true
                }
            })
            .collect();

        result
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl<'src> {
    pub name: Token<'src>,
    pub field_type: FieldType<'src>,
    pub attributes: Vec<Attribute<'src>>,
}

impl<'src> FieldDecl<'src> {
    /// Is this an auto-generated id field ?
    pub fn is_auto_gen_id(&self) -> bool {
        if self.has_id_attrib() {
            self.default_attribute().map_or(false, |default_attrib| {
                default_attrib
                    .arg
                    .as_ref()
                    .map_or(false, |attrib_arg| match attrib_arg {
                        AttribArg::Function(fn_name) => {
                            if let Token::Ident(
                                semantic_analysis::ATTRIB_ARG_FN_AUTO,
                                _span,
                            ) = fn_name
                            {
                                true
                            } else {
                                false
                            }
                        }
                        _ => false,
                    })
            })
        } else {
            false
        }
    }
    /// Returns true if this field has @id attribute.
    pub fn has_id_attrib(&self) -> bool {
        self.get_attribute(semantic_analysis::ATTRIB_NAME_ID)
            .is_some()
    }
    /// Returns true if this field has @unique attribute.
    pub fn has_unique_attrib(&self) -> bool {
        self.get_attribute(semantic_analysis::ATTRIB_NAME_UNIQUE)
            .is_some()
    }
    pub fn has_default_attrib(&self) -> bool {
        self.default_attribute().is_some()
    }
    pub fn default_attribute(&self) -> Option<&Attribute<'src>> {
        self.get_attribute(semantic_analysis::ATTRIB_NAME_DEFAULT)
    }

    #[inline]
    fn get_attribute(&self, attrib_ident_name: &str) -> Option<&Attribute<'src>> {
        self.attributes
            .iter()
            .filter(|attrib| {
                if let Token::Ident(ident_name_str, _span) = attrib.name {
                    attrib_ident_name == ident_name_str
                } else {
                    false
                }
            })
            .next()
    }
}

/// Field type modifier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldTypeMod {
    /// Optional field type modifier, marks the field as optional.
    Optional,
    /// Non-optional field type modifier, marks the field as mandatory.
    NonOptional,
    /// Array field type modifier, marks the field type as array.
    Array,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldType<'src> {
    r#type: RefCell<Type<'src>>, // Note: interier mutability, this is because the field_type for custom types set to Type::Unknown in the first pass. And then in the later pass actual type is determined.
    pub type_mod: FieldTypeMod,
}

impl<'src> FieldType<'src> {
    pub fn is_optional(&self) -> bool {
        if let FieldTypeMod::Optional = self.type_mod {
            true
        } else {
            false
        }
    }
    pub fn is_array(&self) -> bool {
        if let FieldTypeMod::Array = self.type_mod {
            true
        } else {
            false
        }
    }

    pub fn new(r#type: Type<'src>, type_mod: FieldTypeMod) -> FieldType<'src> {
        FieldType {
            r#type: RefCell::new(r#type),
            type_mod,
        }
    }
    /// Is this a scalar (i.e. non-array) short string type ?
    pub fn is_scalar_short_str(&self) -> bool {
        if self.is_array() {
            false
        } else {
            match &*self.r#type() {
                Type::Primitive {
                    r#type: PrimitiveType::ShortStr,
                    ..
                } => true,
                _ => false,
            }
        }
    }
    /// Is this typed as a  scalar field (i.e) can it hold only one value ?
    /// **Note**: If this is an array type, this field is able to
    /// hold more than one value. Hence it is not scalar field.
    pub fn is_scalar(&self) -> bool {
        if self.is_array() {
            false
        } else {
            match &*self.r#type() {
                Type::Primitive { .. } | Type::Enum { .. } => true,
                _ => false,
            }
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
    Enum {
        enum_ty_name: Token<'src>,
    },
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
            Self::Enum { enum_ty_name } => enum_ty_name,
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

    pub fn scalar_field_name(&self) -> Option<&Token<'src>> {
        match self {
            Self::OneSideRelation { .. } => None,
            Self::OneSideRelationRight {
                scalar_field_name, ..
            } => Some(scalar_field_name),
            Self::ManySideRelation {
                scalar_field_name, ..
            } => Some(scalar_field_name),
            Self::SelfOneToOneRelation {
                scalar_field_name, ..
            } => Some(scalar_field_name),
        }
    }

    pub fn referenced_model_name(&self) -> &Token<'src> {
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

    pub fn referenced_model_field_name(&self) -> Option<&Token<'src>> {
        match self {
            Self::OneSideRelation { .. } => None,
            Self::OneSideRelationRight {
                referenced_field_name,
                ..
            } => Some(referenced_field_name),
            Self::ManySideRelation {
                referenced_field_name,
                ..
            } => Some(referenced_field_name),
            Self::SelfOneToOneRelation {
                referenced_field_name,
                ..
            } => Some(referenced_field_name),
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

impl<'src> std::fmt::Display for AttribArg<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttribArg::Args(args) => {
                let disp_str = args.iter().fold("".to_string(), |acc, arg| {
                    format!(
                        "{} {}:{}",
                        acc,
                        arg.arg_name.ident_name().unwrap(),
                        // Note: arg_value should be either an ident or a string, and nothing else.
                        arg.arg_value.ident_name().or(arg.arg_value.str()).unwrap()
                    )
                });
                write!(f, "{}", disp_str)
            }
            AttribArg::Function(fn_name) => {
                write!(f, "{}", fn_name.ident_name().unwrap())
            }
            AttribArg::Ident(v) => {
                write!(f, "{}", v.ident_name().unwrap())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamedArg<'src> {
    pub arg_name: Token<'src>,
    pub arg_value: Token<'src>,
}
