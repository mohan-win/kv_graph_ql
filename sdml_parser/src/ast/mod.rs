//! Abstract Synctax Tree (AST) types of Simple Data Modeling Language (SDML).
use std::{borrow::Borrow, collections::HashMap, ops::Deref, sync::Arc};

use crate::parser::semantic_analysis;
use chumsky::span::SimpleSpan;

pub type Span = SimpleSpan<usize>;

/// A thread safe string slice.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Str(Arc<str>);

impl Str {
  pub fn new(str: impl AsRef<str>) -> Self {
    Self(str.as_ref().into())
  }

  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl AsRef<str> for Str {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl Borrow<str> for Str {
  fn borrow(&self) -> &str {
    &self.0
  }
}

impl Deref for Str {
  type Target = str;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl std::fmt::Display for Str {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(&self.0, f)
  }
}

impl PartialEq<String> for Str {
  fn eq(&self, other: &String) -> bool {
    self.as_str() == other
  }
}

impl PartialEq<str> for Str {
  fn eq(&self, other: &str) -> bool {
    self.as_ref() == other
  }
}

impl PartialEq<Str> for String {
  fn eq(&self, other: &Str) -> bool {
    self == other.as_ref()
  }
}

impl PartialEq<Str> for str {
  fn eq(&self, other: &Str) -> bool {
    self == other.as_ref()
  }
}

impl<'a> PartialEq<&'a str> for Str {
  fn eq(&self, other: &&'a str) -> bool {
    self.as_ref() == *other
  }
}

impl PartialEq<Str> for &'_ str {
  fn eq(&self, other: &Str) -> bool {
    *self == other.as_ref()
  }
}

#[derive(Debug, Clone)]
pub enum Token {
  Ident(Str, Span),
  String(Str, Span),
  Int(i64, Span),
  Float(f64, Span),
  Bool(bool, Span),
}
impl PartialEq for Token {
  // Important Note: Implementing Partial equal for Token, which only compares the
  // actual token and doesn't compare the location (span) in which it is found.
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Token::Ident(self_ident, _), Token::Ident(other_ident, _)) => {
        self_ident == other_ident
      }
      (Token::String(self_str, _), Token::String(other_str, _)) => self_str == other_str,
      (Token::Int(self_int, _), Token::Int(other_int, _)) => self_int == other_int,
      (Token::Float(self_float, _), Token::Float(other_float, _)) => {
        self_float == other_float
      }
      (Token::Bool(self_bool, _), Token::Bool(other_bool, _)) => self_bool == other_bool,
      (_, _) => false,
    }
  }
}

impl Token {
  /// If the token is an identifier, returns its name
  /// if not, it returns None
  pub fn ident_name(&self) -> Option<String> {
    if let Token::Ident(name, _) = self {
      Some(name.to_string())
    } else {
      None
    }
  }
  pub fn str(&self) -> Option<String> {
    if let Token::String(str, _) = self {
      Some(str.trim_matches('"').to_string())
    } else {
      None
    }
  }
  pub fn span(&self) -> Span {
    match self {
      Token::Ident(_, sp) => *sp,
      Token::String(_, sp) => *sp,
      Token::Int(_, sp) => *sp,
      Token::Float(_, sp) => *sp,
      Token::Bool(_, sp) => *sp,
    }
  }
  pub fn try_get_ident_name(&self) -> Result<&str, (&'static str, Span)> {
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DataModel {
  /// Map of config name to its declarations.
  configs: HashMap<String, ConfigDecl>,
  /// Map of enum name to its declarations.
  enums: HashMap<String, EnumDecl>,
  /// Map of model name to its declarations.
  models: HashMap<String, ModelDecl>,
  /// Map of valid relations with fully formed edges.
  /// Available only after semantic_analysis phase.
  relations: HashMap<String, (RelationEdge, Option<RelationEdge>)>,
}

impl DataModel {
  pub fn new(
    configs: HashMap<String, ConfigDecl>,
    enums: HashMap<String, EnumDecl>,
    models: HashMap<String, ModelDecl>,
    relations: HashMap<String, (RelationEdge, Option<RelationEdge>)>,
  ) -> DataModel {
    DataModel {
      configs,
      enums,
      models,
      relations,
    }
  }
  pub fn configs(&self) -> &HashMap<String, ConfigDecl> {
    &self.configs
  }
  /// Configs sorted by its name in chronogical order.
  pub fn configs_sorted(&self) -> Vec<&ConfigDecl> {
    let mut config_names = self.enums.keys().collect::<Vec<&String>>();
    config_names.sort();
    config_names
      .into_iter()
      .rfold(Vec::new(), |mut acc, config_name| {
        acc.push(self.configs.get(config_name).unwrap());
        acc
      })
  }

  pub fn enums(&self) -> &HashMap<String, EnumDecl> {
    &self.enums
  }
  /// Enums sorted by its name in chronogical order.
  pub fn enums_sorted(&self) -> Vec<&EnumDecl> {
    let mut enum_names = self.enums.keys().collect::<Vec<&String>>();
    enum_names.sort();
    enum_names
      .into_iter()
      .rfold(Vec::new(), |mut acc, enum_name| {
        acc.push(self.enums.get(enum_name).unwrap());
        acc
      })
  }
  pub fn models(&self) -> &HashMap<String, ModelDecl> {
    &self.models
  }
  /// Models sorted by its name in chronogical order.
  pub fn models_sorted(&self) -> Vec<&ModelDecl> {
    let mut model_names = self.models.keys().collect::<Vec<&String>>();
    model_names.sort();
    model_names
      .into_iter()
      .rfold(Vec::new(), |mut acc, model_name| {
        acc.push(self.models.get(model_name).unwrap());
        acc
      })
  }
  pub fn relations(&self) -> &HashMap<String, (RelationEdge, Option<RelationEdge>)> {
    &self.relations
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
  Config(ConfigDecl),
  Enum(EnumDecl),
  Model(ModelDecl),
}

impl Declaration {
  pub fn name(&self) -> &Token {
    match self {
      Declaration::Config(c) => &c.name,
      Declaration::Enum(e) => &e.name,
      Declaration::Model(m) => &m.name,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigDecl {
  pub name: Token,
  pub config_pairs: Vec<ConfigPair>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigPair {
  pub name: Token,
  pub value: ConfigValue,
}

#[derive(Debug, Clone)]
pub enum ConfigValue {
  String(Str, Span),
  Int(i64, Span),
  Float(f64, Span),
  Bool(bool, Span),
}

impl PartialEq for ConfigValue {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (ConfigValue::String(self_str, _), ConfigValue::String(other_str, _)) => {
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

impl TryFrom<Token> for ConfigValue {
  type Error = String;
  fn try_from(value: Token) -> Result<Self, Self::Error> {
    match value {
      Token::Bool(b, s) => Ok(ConfigValue::Bool(b, s)),
      Token::Float(f, s) => Ok(ConfigValue::Float(f, s)),
      Token::Int(i, s) => Ok(ConfigValue::Int(i, s)),
      Token::String(str, s) => Ok(ConfigValue::String(str, s)),
      t => Err(format!("Token {:?} can't be turned into ConfigValue", t)),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDecl {
  pub name: Token,
  pub elements: Vec<Token>,
}

/// Represents an entity inside the application domain.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelDecl {
  pub name: Token,
  pub fields: Vec<FieldDecl>,
}

#[derive(Debug, Clone)]
pub struct ModelFields<'a> {
  pub relation: Vec<&'a FieldDecl>,
  /// relation_scalar: Vec<(field, is_unique_or_indexed)>
  relation_scalar: Vec<(&'a FieldDecl, bool)>,
  /// id: Vec<(field, is_auto_generated)>
  pub id: Vec<(&'a FieldDecl, bool)>,
  pub unique: Vec<&'a FieldDecl>,
  /// rest: Vec<(field, is_indexed)>
  rest: Vec<(&'a FieldDecl, bool)>,
}
pub enum ModelIndexedFieldsFilter {
  All,
  OnlyIndexedFields,
  OnlyNonIndexedFields,
}

impl<'a> ModelFields<'a> {
  /// Retrieves all of the indexed fields of the model
  /// Following fields are indexed in the data store.
  /// * Field with @id attribute
  /// * Field with @unique attribute
  /// * Field with explicit @indexed attribute.
  pub fn all_indexed(&self) -> Vec<&FieldDecl> {
    let mut indexed_fields = Vec::new();
    indexed_fields.extend(self.id.iter().map(|(field, _is_auto_gen)| field));
    indexed_fields.extend(self.unique.iter());
    indexed_fields.extend(self.relation_scalar.iter().filter_map(
      |(field, is_unique_or_indexed)| {
        if *is_unique_or_indexed {
          Some(field)
        } else {
          None
        }
      },
    ));
    indexed_fields.extend(self.rest.iter().filter_map(|(field, is_indexed)| {
      if *is_indexed {
        Some(field)
      } else {
        None
      }
    }));

    indexed_fields
  }

  /// Get Relation scalar fields.
  pub fn get_relation_scalars(
    &self,
    filter: ModelIndexedFieldsFilter,
  ) -> Vec<&FieldDecl> {
    self
      .relation_scalar
      .iter()
      .filter_map(|(field, is_unique_or_indexed)| match filter {
        ModelIndexedFieldsFilter::All => Some(*field),
        ModelIndexedFieldsFilter::OnlyIndexedFields if *is_unique_or_indexed => {
          Some(*field)
        }
        ModelIndexedFieldsFilter::OnlyNonIndexedFields if !*is_unique_or_indexed => {
          Some(*field)
        }
        _ => None,
      })
      .collect()
  }

  pub fn get_rest(&self, filter: ModelIndexedFieldsFilter) -> Vec<&FieldDecl> {
    self
      .rest
      .iter()
      .filter_map(|(field, is_indexed)| match filter {
        ModelIndexedFieldsFilter::All => Some(*field),
        ModelIndexedFieldsFilter::OnlyIndexedFields if *is_indexed => Some(*field),
        ModelIndexedFieldsFilter::OnlyNonIndexedFields if !*is_indexed => Some(*field),
        _ => None,
      })
      .collect()
  }
}

impl ModelDecl {
  /// Get Model fields.
  pub fn get_fields(&self) -> ModelFields {
    self.get_fields_internal(false) // IMPORTNAT: Don't allow unknown_field_type when fields are accessed from outside the crate.
  }

  pub(crate) fn get_fields_internal(
    &self,
    allow_unknown_field_type: bool,
  ) -> ModelFields {
    let mut result = ModelFields {
      relation: Vec::new(),
      relation_scalar: Vec::new(),
      id: Vec::new(),
      unique: Vec::new(),
      rest: Vec::new(),
    };

    let mut relation_scalar_field_names = Vec::new();
    self
      .fields
      .iter()
      .for_each(|field| match &field.field_type.r#type {
        Type::Unknown(..) => {
          if !allow_unknown_field_type {
            panic!("Can't allow unknown field type.")
          }
        }
        Type::Relation(edge) => {
          result.relation.push(&field);
          edge.scalar_field_name().map(|fld_name| {
            relation_scalar_field_names.push(fld_name.ident_name().unwrap().to_string())
          });
        }
        Type::Primitive { .. } | Type::Enum { .. } => {
          if field.is_auto_gen_id() {
            result.id.push((field, true));
          } else if field.has_id_attrib() {
            result.id.push((field, false));
          } else if field.has_unique_attrib() {
            result.unique.push(field);
          } else {
            result.rest.push((field, field.has_indexed_attrib()));
          }
        }
      });

    let mut relation_scalar_fields = Vec::new();
    // Filter-out relation scalar fields from unique & non-unique fields.
    result.unique = result
      .unique
      .into_iter()
      .filter(|field| {
        if relation_scalar_field_names
          .contains(&field.name.ident_name().unwrap().to_string())
        {
          // Note: relation scalar fields with @unique attribute are indexed.
          // Hence setting is_unique_or_indexed to true.
          relation_scalar_fields.push((*field, true));
          false
        } else {
          true
        }
      })
      .collect();
    result.rest = result
      .rest
      .into_iter()
      .filter(|(field, is_indexed)| {
        if relation_scalar_field_names
          .contains(&field.name.ident_name().unwrap().to_string())
        {
          relation_scalar_fields.push((*field, *is_indexed));
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
pub struct FieldDecl {
  pub name: Token,
  pub field_type: FieldType,
  pub attributes: Vec<Attribute>,
}

impl FieldDecl {
  /// Is this an auto-generated id field ?
  pub fn is_auto_gen_id(&self) -> bool {
    if self.has_id_attrib() {
      self.default_attribute().map_or(false, |default_attrib| {
        default_attrib
          .arg
          .as_ref()
          .map_or(false, |attrib_arg| match attrib_arg {
            AttribArg::Function(fn_name) => {
              if let Token::Ident(ident_name, _) = fn_name {
                ident_name == semantic_analysis::ATTRIB_ARG_FN_AUTO
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
    self
      .get_attribute(semantic_analysis::ATTRIB_NAME_ID)
      .is_some()
  }
  /// Returns true if this field has @unique attribute.
  pub fn has_unique_attrib(&self) -> bool {
    self
      .get_attribute(semantic_analysis::ATTRIB_NAME_UNIQUE)
      .is_some()
  }
  pub fn has_indexed_attrib(&self) -> bool {
    self
      .get_attribute(semantic_analysis::ATTRIB_NAME_INDEXED)
      .is_some()
  }
  pub fn has_default_attrib(&self) -> bool {
    self.default_attribute().is_some()
  }
  pub fn default_attribute(&self) -> Option<&Attribute> {
    self.get_attribute(semantic_analysis::ATTRIB_NAME_DEFAULT)
  }

  #[inline]
  fn get_attribute(&self, attrib_ident_name: &str) -> Option<&Attribute> {
    self
      .attributes
      .iter()
      .filter(|attrib| {
        if let Token::Ident(ident_name_str, _span) = &attrib.name {
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
pub struct FieldType {
  r#type: Type,
  pub type_mod: FieldTypeMod,
}

impl FieldType {
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

  pub fn new(r#type: Type, type_mod: FieldTypeMod) -> FieldType {
    FieldType { r#type, type_mod }
  }

  /// Is this a scalar (i.e. non-array) short string type ?
  pub fn is_scalar_short_str(&self) -> bool {
    if self.is_array() {
      false
    } else {
      matches!(
        self.r#type,
        Type::Primitive {
          r#type: PrimitiveType::ShortStr,
          ..
        }
      )
    }
  }

  /// Is this typed as a  scalar field (i.e) can it hold only one value ?
  /// **Note**: If this is an array type, this field is able to
  /// hold more than one value. Hence it is not scalar field.
  /// Also relation fields are non-scalar fields.
  pub fn is_scalar(&self) -> bool {
    if self.is_array() {
      false
    } else {
      matches!(self.r#type, Type::Primitive { .. } | Type::Enum { .. })
    }
  }

  pub fn r#type(&self) -> &Type {
    &self.r#type
  }

  pub(crate) fn set_type(&mut self, new_type: Type) {
    self.r#type = new_type;
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
  Primitive {
    r#type: PrimitiveType,
    token: Token,
  },
  Enum {
    enum_ty_name: Token,
  },
  /// If field type is other model type, then its a `Relation`.
  Relation(RelationEdge),
  /// If the field type is Enum or Relation, in the first pass it will be set to Unknown with identifier token.
  /// Then only during scemantic analysis its actual user defined type is determined.
  Unknown(Token),
}

impl Type {
  pub fn token(&self) -> &Token {
    match self {
      Self::Primitive { token, .. } => token,
      Self::Enum { enum_ty_name } => enum_ty_name,
      Self::Relation(relation_edge) => relation_edge.referenced_model_name(),
      Self::Unknown(token) => token,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationEdge {
  /// One-side of the relation, for both
  /// Left side of the 1-to-1 relation and
  /// One side of the 1-to-many relation.
  OneSideRelation {
    relation_name: Token,
    referenced_model_name: Token,
  },
  /// One-side of the relation on the right in 1-to-1 relation.
  OneSideRelationRight {
    relation_name: Token,
    /// Name of the relational scalar field storing the foreign key values.
    /// It should be marked with @unique attribute, to make (1-to-1) explicit in the schema.
    scalar_field_name: Token,
    referenced_model_name: Token,
    /// Name of the field (should be either @id or @unique) in the referenced model.
    referenced_field_name: Token,
  },
  /// Many-side of the relation, capturing the required information for
  /// a. Many side of 1-to-many relation,
  /// b. Both sides fo the many-to-many relation.
  /// c. Many side of the self-to-many relation,
  ManySideRelation {
    relation_name: Token,
    /// Name of the relational scalar field storing the foreign key values.
    scalar_field_name: Token,
    referenced_model_name: Token,
    /// Name of the field (should be either @id or @unique) in the referenced model.
    referenced_field_name: Token,
  },
  /// Self relation of type 1-to-1
  SelfOneToOneRelation {
    relation_name: Token,
    /// Name of the scalar field name. It should be marked with @unique attribute, to make (1-to-1) explicit in the schema.
    scalar_field_name: Token,
    referenced_model_name: Token,
    /// Name of the referened field (should be either @id or @unique) in the model.
    referenced_field_name: Token,
  },
}

impl RelationEdge {
  pub fn relation_name(&self) -> &Token {
    match self {
      Self::OneSideRelation { relation_name, .. } => relation_name,
      Self::OneSideRelationRight { relation_name, .. } => relation_name,
      Self::ManySideRelation { relation_name, .. } => relation_name,
      Self::SelfOneToOneRelation { relation_name, .. } => relation_name,
    }
  }

  pub fn scalar_field_name(&self) -> Option<&Token> {
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

  pub fn referenced_model_name(&self) -> &Token {
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

  pub fn referenced_model_field_name(&self) -> Option<&Token> {
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
pub struct Attribute {
  pub name: Token,
  pub arg: Option<AttribArg>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttribArg {
  Args(Vec<NamedArg>),
  Function(Token),
  Ident(Token),
}

impl std::fmt::Display for AttribArg {
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
pub struct NamedArg {
  pub arg_name: Token,
  pub arg_value: Token,
}
