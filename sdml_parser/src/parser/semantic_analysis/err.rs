use core::fmt;

use crate::ast::Span;

/// Type of the semantic error
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticError<'src> {
    /// This error is returned when name of a user defined type clashes with already existing type.
    DuplicateTypeDefinition { span: Span, type_name: &'src str },
    /// This error is returned if type of a field is undefined.
    UndefinedType {
        span: Span,
        type_name: &'src str,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// This error is returned enum type used is undefined.
    UndefinedEnum {
        span: Span,
        r#enum: &'src str,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// This error is returned if undefined enum value is used.
    UndefinedEnumValue {
        span: Span,
        enum_value: &'src str,
        attrib_name: &'src str,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// Invalid Relation - This error is thrown for invalid relation.
    RelationInvalid {
        span: Span,
        relation_name: &'src str,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// Duplicate Relation - This error is thrown when *same* relation name represent more than one relation.
    RelationDuplicate {
        span: Span,
        relation_name: &'src str,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// Partial Relation - This error is thrown if only one edge is present for a relation.
    RelationPartial {
        span: Span,
        relation_name: &'src str,
        field_name: Option<&'src str>,
        model_name: Option<&'src str>,
    },
    /// This error is thrown if relation attribute is not there
    /// on either side of the relation.
    RelationNoAttribute {
        span: Span,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// This error is thrown if the relation scalar field is not found.
    RelationScalarFieldNotFound {
        span: Span,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// This error is thrown if the relation scalar field referenced in relation
    /// attribute is invalid.
    RelationScalarFieldIsNotScalar {
        span: Span,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// This error is thrown if the relation scalar field of array type is marked with
    /// UNIQUE attribute.
    RelationScalarFieldArrayCanNotBeUnique {
        span: Span,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// This error is thrown if the referenced field in the relation
    /// attribute is not found in the referenced model.
    RelationReferencedFieldNotFound {
        span: Span,
        field_name: &'src str,
        model_name: &'src str,
        referenced_field_name: &'src str,
        referenced_model_name: &'src str,
    },
    /// This error is thrown if the referenced field in a relation attribute is not
    /// scalar field in the referenced model.
    RelationReferencedFieldNotScalar {
        span: Span,
        field_name: &'src str,
        model_name: &'src str,
        referenced_field_name: &'src str,
        referenced_model_name: &'src str,
    },
    /// This error is thrown if the referenced field in a relation attribute is not
    /// unique or id field in the referenced model.
    RelationReferencedFieldNotUnique {
        span: Span,
        field_name: &'src str,
        model_name: &'src str,
        referenced_field_name: &'src str,
        referenced_model_name: &'src str,
    },
    /// This error is thrown for relation attribute which is missing name.
    RelationAttributeMissingName {
        span: Span,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// This error is thrown if any attribute on a relation is invalid.
    RelationInvalidAttribute {
        span: Span,
        attrib_name: &'src str,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// This error is thrown if arguments to relation attribute is invalid.
    RelationInvalidAttributeArg {
        span: Span,
        relation_name: Option<&'src str>,
        field_name: Option<&'src str>,
        model_name: Option<&'src str>,
    },

    /// This error is thrown if the attribute is invalid.
    AttributeInvalid {
        span: Span,
        attrib_name: &'src str,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// This error is thrown if a model field has more than 1 attribute, and they are incompatible with each other
    AttributeIncompatible {
        span: Span,
        attrib_name: &'src str,
        /// First attribute present in the field.
        first_attrib_name: &'src str,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// This error is returned for unknown attribute usage in model's fields.
    AttributeUnknown {
        span: Span,
        attrib_name: &'src str,
        field_name: &'src str,
        model_name: &'src str,
    },
    /// This error is thrown if the argment passed to attribute is invalid.
    AttributeArgInvalid {
        span: Span,
        attrib_arg_name: Option<&'src str>,
        attrib_name: &'src str,
        field_name: &'src str,
        model_name: &'src str,
    },
}

impl<'src> fmt::Display for SemanticError<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Semantic Error: {self:#?}")
    }
}
