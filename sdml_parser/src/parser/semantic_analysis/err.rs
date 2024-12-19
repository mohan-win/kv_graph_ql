use chumsky::error::Rich as ChumskyError;
use core::fmt;

use crate::types::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
  ParserError {
    span: Span,
    message: String,
  },
  /// This error is returned when a Model is missing an Id field (a.k.a field marked with @id attribute).
  ModelIdFieldMissing {
    span: Span,
    model_name: String,
  },
  /// This error is thrown when a Model has more than one field marked with @id attribute.
  ModelIdFieldDuplicate {
    span: Span,
    field_name: String,
    model_name: String,
  },
  /// This error is thrown when a Model doesn't stores any data.
  ModelEmpty {
    span: Span,
    model_name: String,
  },
  /// This error is returned when name of a user defined type clashes with already existing type.
  TypeDuplicateDefinition {
    span: Span,
    type_name: String,
  },
  /// This error is returned if type of a field is undefined.
  TypeUndefined {
    span: Span,
    type_name: String,
    field_name: String,
    model_name: String,
  },
  /// This error is returned if undefined enum value is used.
  EnumValueUndefined {
    span: Span,
    enum_value: String,
    attrib_name: String,
    field_name: String,
    model_name: String,
  },
  /// This error is thrown if the attribute is invalid.
  AttributeInvalid {
    span: Span,
    reason: String,
    attrib_name: String,
    field_name: String,
    model_name: String,
  },
  /// This error is thrown if a model field has more than 1 attribute, and they are incompatible with each other
  AttributeIncompatible {
    span: Span,
    attrib_name: String,
    /// First attribute present in the field.
    first_attrib_name: String,
    field_name: String,
    model_name: String,
  },
  /// This error is returned for unknown attribute usage in model's fields.
  AttributeUnknown {
    span: Span,
    attrib_name: String,
    field_name: String,
    model_name: String,
  },
  /// This error is thrown if the argment passed to attribute is invalid.
  AttributeArgInvalid {
    span: Span,
    attrib_arg_name: Option<String>,
    attrib_name: String,
    field_name: String,
    model_name: String,
  },
  /// Invalid Relation - This error is thrown for invalid relation.
  RelationInvalid {
    span: Span,
    relation_name: String,
    field_name: Option<String>,
    model_name: Option<String>,
  },
  /// Duplicate Relation - This error is thrown when *same* relation name represent more than one relation.
  RelationDuplicate {
    span: Span,
    relation_name: String,
    field_name: String,
    model_name: String,
  },
  /// Partial Relation - This error is thrown if only one edge is present for a relation.
  RelationPartial {
    span: Span,
    relation_name: String,
    field_name: Option<String>,
    model_name: Option<String>,
  },
  /// This error is thrown if relation attribute is not there
  /// on either side of the relation.
  RelationAttributeMissing {
    span: Span,
    field_name: String,
    model_name: String,
  },
  /// This error is thrown if the relation scalar field is not found.
  RelationScalarFieldNotFound {
    span: Span,
    scalar_field_name: Option<String>,
    field_name: String,
    model_name: String,
  },
  /// This error is thrown if the relation scalar field referenced in relation
  /// attribute is not of primitive type.
  RelationScalarFieldIsNotPrimitive {
    span: Span,
    field_name: String,
    model_name: String,
  },
  /// This error is thrown if the referenced field in the relation
  /// attribute is not found in the referenced model.
  RelationReferencedFieldNotFound {
    span: Span,
    field_name: String,
    model_name: String,
    referenced_field_name: String,
    referenced_model_name: String,
  },
  /// This error is thrown if the referenced field in a relation attribute is not
  /// scalar field in the referenced model.
  RelationReferencedFieldNotScalar {
    span: Span,
    field_name: String,
    model_name: String,
    referenced_field_name: String,
    referenced_model_name: String,
  },
  /// This error is thrown if the referenced field in a relation attribute is not
  /// unique or id field in the referenced model.
  RelationReferencedFieldNotUnique {
    span: Span,
    field_name: String,
    model_name: String,
    referenced_field_name: String,
    referenced_model_name: String,
  },
  /// This error is thrown when relation scalar field is not unique,
  /// Ex. In an 1-to-1 relation, relation scalar field should be unique.
  RelationScalarFieldNotUnique {
    /// span of relation scalar field.
    span: Span,
    /// relation scalar field name.
    field_name: String,
    model_name: String,
    referenced_model_name: String,
    referenced_model_relation_field_name: Option<String>,
  },
  /// This error is thrown when relation scalar field is unique,
  /// Ex. In an 1-to-Many relation, the relation scalar field shouldn't be unique.
  RelationScalarFieldIsUnique {
    /// span of relation scalar field.
    span: Span,
    /// relation scalar field name.
    field_name: String,
    model_name: String,
    referenced_model_name: String,
    referenced_model_relation_field_name: String,
  },
  /// This error is thrown when relation scalar and
  /// referenced fields in a relation, has mismatching types.
  RelationScalarAndReferencedFieldsTypeMismatch {
    span: Span,
    field_name: String,
    model_name: String,
    referenced_field_name: String,
    referenced_model_name: String,
  },
  /// This error is thrown if any attribute on a relation is invalid.
  RelationInvalidAttribute {
    span: Span,
    attrib_name: String,
    field_name: String,
    model_name: String,
  },
  /// This error is thrown if arguments to relation attribute is invalid.
  RelationInvalidAttributeArg {
    span: Span,
    relation_name: Option<String>,
    arg_name: Option<String>,
    field_name: Option<String>,
    model_name: Option<String>,
  },
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Semantic Error: {self:#?}")
  }
}

impl<'a, T: fmt::Display> From<ChumskyError<'a, T>> for Error {
  fn from(value: ChumskyError<'a, T>) -> Self {
    Error::ParserError {
      span: *value.span(),
      message: value.reason().to_string(),
    }
  }
}
