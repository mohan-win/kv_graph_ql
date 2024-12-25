use crate::types::{DataModel, Declaration, DeclarationsGrouped};
use std::collections::{HashMap, HashSet};

mod attribute;
/// Error Module
pub mod err;
mod relation;
mod visitor;
mod visitors;

pub(crate) use attribute::ATTRIB_ARG_FN_AUTO;
pub(crate) use attribute::ATTRIB_NAME_DEFAULT;
pub(crate) use attribute::ATTRIB_NAME_ID;
pub(crate) use attribute::ATTRIB_NAME_INDEXED;
pub(crate) use attribute::ATTRIB_NAME_UNIQUE;

use err::Error;
use relation::RelationMap;

/// This function performs semantic analysis, converts parsed declarations into `DataModel`
/// if no errors found. In case errors are found during semantic analyis, it returns the errors.
pub(crate) fn semantic_update(
  declarations: Vec<Declaration>,
) -> Result<DataModel, Vec<Error>> {
  let declarations = categorise_declarations(declarations)?;

  let mut build_visitors =
    visitor::VisitorNil.with(visitors::UpdateUnknownFields::default());

  // Build the data_model.
  let data_model = visitor::build_data_model(&mut build_visitors, &declarations)?;

  let mut validate_visitors = visitor::VisitorNil
    .with(visitors::ValidateModelHasIdField)
    .with(visitors::ValidateFieldAttributes)
    .with(visitors::ValidateFieldAttribute)
    .with(visitors::ValidateAttributeArgs);

  // Validate the data_model.
  visitor::validate_data_model(&mut validate_visitors, &data_model)?;

  Ok(data_model)
}

fn categorise_declarations(
  declarations: Vec<Declaration>,
) -> Result<DeclarationsGrouped, Vec<Error>> {
  let mut errs: Vec<Error> = Vec::new();
  let mut type_set: HashSet<String> = HashSet::new();

  let mut configs = HashMap::new();
  let mut enums = HashMap::new();
  let mut models = HashMap::new();

  for decl in declarations.into_iter() {
    let (type_name, span) = match decl {
      Declaration::Config(c) => {
        let type_name = c.name.ident_name().unwrap();
        let span = c.name.span();
        configs.insert(type_name.clone(), c);
        (type_name, span)
      }
      Declaration::Enum(e) => {
        let type_name = e.name.ident_name().unwrap();
        let span = e.name.span();
        enums.insert(type_name.clone(), e);
        (type_name, span)
      }
      Declaration::Model(m) => {
        let type_name = m.name.ident_name().unwrap();
        let span = m.name.span();
        models.insert(type_name.clone(), m);
        (type_name, span)
      }
    };

    if type_set.contains(&type_name) {
      errs.push(Error::TypeDuplicateDefinition { span, type_name });
    } else {
      type_set.insert(type_name);
    }
  }

  if errs.is_empty() {
    Ok(DeclarationsGrouped {
      configs,
      enums,
      models,
    })
  } else {
    Err(errs)
  }
}

#[cfg(test)]
mod tests {
  use crate::types::Span;

  use super::*;
  use chumsky::prelude::*;

  #[test]
  fn test_duplicate_types() {
    let duplicate_types_sdml = std::fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/semantic_analysis/duplicate_types.sdml"
    ))
    .unwrap();
    let decls = crate::parser::delcarations()
      .parse(&duplicate_types_sdml)
      .into_result()
      .unwrap();

    match semantic_update(decls) {
      Ok(_) => assert!(false, "Model file with duplicate types should throw err!"),
      Err(errs) => {
        assert_eq!(
          vec![
            Error::TypeDuplicateDefinition {
              span: Span::new(52, 54),
              type_name: "db".to_string()
            },
            Error::TypeDuplicateDefinition {
              span: Span::new(294, 311),
              type_name: "User".to_string()
            },
            Error::TypeDuplicateDefinition {
              span: Span::new(666, 670),
              type_name: "Role".to_string()
            }
          ],
          errs
        )
      }
    }
  }

  #[test]
  fn test_model_errs() {
    let model_errs_sdml = std::fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/semantic_analysis/model_errs.sdml"
    ))
    .unwrap();
    let expected_semantic_errs: Vec<Error> = vec![
      Error::ModelIdFieldDuplicate {
        span: Span::new(750, 762),
        field_name: "name".to_string(),
        model_name: "Category".to_string(),
      },
      Error::ModelIdFieldMissing {
        span: Span::new(45, 291),
        model_name: "User".to_string(),
      },
      Error::AttributeInvalid {
        span: Span::new(464, 467),
        reason: "Only Non-Optional Scalar Short String field is allowed".to_string(),
        attrib_name: "id".to_string(),
        field_name: "postId".to_string(),
        model_name: "Post".to_string(),
      },
      Error::ModelEmpty {
        span: Span::new(849, 872),
        model_name: "EmptyModel".to_string(),
      },
      Error::ModelEmpty {
        span: Span::new(872, 948),
        model_name: "EmptyModelOnlyAutoGenId".to_string(),
      },
      Error::AttributeInvalid {
        span: Span::new(337, 340),
        reason: "Only Non-Optional Scalar Short String field is allowed".to_string(),
        attrib_name: "id".to_string(),
        field_name: "profileId".to_string(),
        model_name: "Profile".to_string(),
      },
      Error::AttributeInvalid {
        span: Span::new(341, 357),
        reason: "Only Non-Optional Scalar field is allowed".to_string(),
        attrib_name: "default".to_string(),
        field_name: "profileId".to_string(),
        model_name: "Profile".to_string(),
      },
    ];

    let decls = crate::parser::delcarations()
      .parse(&model_errs_sdml)
      .into_result()
      .unwrap();
    match semantic_update(decls) {
      Ok(_) => assert!(false, "Expecting model errors to surface"),
      Err(errs) => {
        assert_eq!(expected_semantic_errs.len(), errs.len());
        errs
          .into_iter()
          .for_each(|e| assert!(expected_semantic_errs.contains(&e)))
      }
    }
  }

  #[test]
  fn test_field_errs() {
    let field_errs_sdml = std::fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/semantic_analysis/field_errs.sdml"
    ))
    .unwrap();
    let expected_semantic_errs: Vec<Error> = vec![
      Error::TypeUndefined {
        span: Span::new(625, 629),
        type_name: "bool".to_string(),
        field_name: "published".to_string(),
        model_name: "Post".to_string(),
      },
      Error::TypeUndefined {
        span: Span::new(246, 251),
        type_name: "Role1".to_string(),
        field_name: "role1".to_string(),
        model_name: "User".to_string(),
      },
    ];

    let field_errs1_sdml = std::fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/semantic_analysis/field_errs1.sdml"
    ))
    .unwrap();
    let expected_semantic_errs1: Vec<Error> = vec![
      Error::AttributeInvalid {
        span: Span::new(640, 655),
        reason: String::from("Only Non-Optional Scalar field is allowed"),
        attrib_name: "default".to_string(),
        field_name: "published".to_string(),
        model_name: "Post".to_string(),
      },
      Error::EnumValueUndefined {
        span: Span::new(223, 228),
        enum_value: "GUEST".to_string(),
        attrib_name: "default".to_string(),
        field_name: "role".to_string(),
        model_name: "User".to_string(),
      },
    ];

    let decls = crate::parser::delcarations()
      .parse(&field_errs_sdml)
      .into_result()
      .unwrap();
    match semantic_update(decls) {
      Ok(_) => assert!(false, "Expecting field errors to surface"),
      Err(errs) => {
        assert_eq!(expected_semantic_errs.len(), errs.len());
        errs
          .into_iter()
          .for_each(|e| assert!(expected_semantic_errs.contains(&e)));
      }
    }

    let decls = crate::parser::delcarations()
      .parse(&field_errs1_sdml)
      .into_result()
      .unwrap();
    match semantic_update(decls) {
      Ok(_) => assert!(false, "Expecting field errors to surface"),
      Err(errs) => {
        assert_eq!(expected_semantic_errs1.len(), errs.len());
        errs
          .into_iter()
          .for_each(|e| assert!(expected_semantic_errs1.contains(&e)));
      }
    }
  }

  #[test]
  fn test_relation_errs_invalid() {
    let relation_errs_sdml = std::fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/semantic_analysis/relation_errs/invalid.sdml"
    ))
    .unwrap();
    let expected_semantic_errs: Vec<Error> = vec![
      Error::RelationScalarFieldNotUnique {
        span: Span::new(169, 182),
        field_name: "spouseUserId".to_string(),
        model_name: "User".to_string(),
        referenced_model_name: "User".to_string(),
        referenced_model_relation_field_name: None,
      },
      Error::RelationScalarFieldIsUnique {
        span: Span::new(335, 344),
        field_name: "authorId".to_string(),
        model_name: "Post".to_string(),
        referenced_model_name: "User".to_string(),
        referenced_model_relation_field_name: "posts".to_string(),
      },
      Error::RelationScalarFieldNotUnique {
        span: Span::new(660, 669),
        field_name: "authorId".to_string(),
        model_name: "Post1".to_string(),
        referenced_model_name: "User1".to_string(),
        referenced_model_relation_field_name: Some("singlePost".to_string()),
      },
      Error::RelationPartial {
        span: Span::new(562, 570),
        relation_name: "posts1".to_string(),
        field_name: None,
        model_name: None,
      },
      Error::RelationPartial {
        span: Span::new(239, 246),
        relation_name: "posts".to_string(),
        field_name: None,
        model_name: None,
      },
      Error::ModelEmpty {
        span: Span::new(454, 575),
        model_name: "User1".to_string(),
      },
    ];

    let decls = crate::parser::delcarations()
      .parse(&relation_errs_sdml)
      .into_result()
      .unwrap();
    match semantic_update(decls) {
      Ok(_) => assert!(false, "Expecting relation errors to surface"),
      Err(errs) => {
        assert_eq!(expected_semantic_errs.len(), errs.len());
        errs
          .into_iter()
          .for_each(|e| assert!(expected_semantic_errs.contains(&e)));
      }
    }
  }

  #[test]
  fn test_relation_errs_duplicate() {
    let relation_errs_sdml = std::fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/semantic_analysis/relation_errs/duplicate.sdml"
    ))
    .unwrap();

    let decls = crate::parser::delcarations()
      .parse(&relation_errs_sdml)
      .into_result()
      .unwrap();
    match semantic_update(decls) {
      Ok(_) => assert!(false, "Expecting relation errors to surface"),
      Err(errs) => {
        eprintln!("{errs:#?}");
        let errs: Vec<_> = errs
          .into_iter()
          .filter(|e| match e {
            Error::RelationDuplicate { .. } => true,
            _ => false,
          })
          .collect();
        assert_eq!(3, errs.len());
      }
    }
  }

  #[test]
  fn test_relation_errs_partial() {
    let relation_errs_sdml = std::fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/semantic_analysis/relation_errs/partial.sdml"
    ))
    .unwrap();
    let expected_semantic_errs: Vec<Error> = vec![
      Error::RelationInvalid {
        span: Span::new(738, 752),
        relation_name: "user_profile".to_string(),
        field_name: Some("user".to_string()),
        model_name: Some("Profile".to_string()),
      },
      Error::RelationInvalid {
        span: Span::new(385, 397),
        relation_name: "user_posts".to_string(),
        field_name: Some("author".to_string()),
        model_name: Some("Post".to_string()),
      },
      Error::RelationInvalid {
        span: Span::new(515, 531),
        relation_name: "negative_posts".to_string(),
        field_name: Some("negativeAuthor".to_string()),
        model_name: Some("Post".to_string()),
      },
      Error::RelationPartial {
        span: Span::new(153, 168),
        relation_name: "mentor_mentee".to_string(),
        field_name: None,
        model_name: None,
      },
    ];

    let decls = crate::parser::delcarations()
      .parse(&relation_errs_sdml)
      .into_result()
      .unwrap();
    match semantic_update(decls) {
      Ok(_) => assert!(false, "Expecting relation errors to surface"),
      Err(errs) => {
        assert_eq!(expected_semantic_errs.len(), errs.len());
        errs
          .into_iter()
          .for_each(|e| assert!(expected_semantic_errs.contains(&e)));
      }
    }
  }

  #[test]
  fn test_relation_errs_attribute_missing() {
    let relation_errs_sdml = std::fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/semantic_analysis/relation_errs/attribute_missing.sdml"
    ))
    .unwrap();
    let expected_semantic_errs: Vec<Error> = vec![
      Error::RelationAttributeMissing {
        span: Span::new(171, 187),
        field_name: "negativePosts".to_string(),
        model_name: "User".to_string(),
      },
      Error::RelationAttributeMissing {
        span: Span::new(465, 481),
        field_name: "spouse".to_string(),
        model_name: "User".to_string(),
      },
      Error::RelationAttributeMissing {
        span: Span::new(1030, 1042),
        field_name: "user".to_string(),
        model_name: "Profile".to_string(),
      },
    ];

    let decls = crate::parser::delcarations()
      .parse(&relation_errs_sdml)
      .into_result()
      .unwrap();
    match semantic_update(decls) {
      Ok(_) => assert!(false, "Expecting relation errors to surface"),
      Err(errs) => {
        eprintln!("{errs:#?}");
        let errs: Vec<_> = errs
          .into_iter()
          .filter(|e| match e {
            Error::RelationAttributeMissing { .. } => true,
            _ => false,
          })
          .collect();
        assert_eq!(expected_semantic_errs.len(), errs.len());
        errs
          .into_iter()
          .for_each(|e| assert!(expected_semantic_errs.contains(&e)));
      }
    }
  }

  #[test]
  fn test_relation_errs_attribute_arg_invalid() {
    let relation_errs_sdml = std::fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/semantic_analysis/relation_errs/attribute_arg_invalid.sdml"
    ))
    .unwrap();
    let expected_semantic_errs: Vec<Error> = vec![
      Error::RelationInvalidAttributeArg {
        span: Span::new(110, 126),
        relation_name: None,
        arg_name: Some("name1".to_string()),
        field_name: Some("posts".to_string()),
        model_name: Some("User".to_string()),
      },
      Error::AttributeArgInvalid {
        span: Span::new(148, 153),
        attrib_arg_name: Some("name1".to_string()),
        attrib_name: "relation".to_string(),
        field_name: "posts".to_string(),
        model_name: "User".to_string(),
      },
      Error::RelationInvalidAttributeArg {
        span: Span::new(495, 511),
        relation_name: None,
        arg_name: None,
        field_name: Some("spouse".to_string()),
        model_name: Some("User".to_string()),
      },
      Error::RelationScalarFieldNotFound {
        span: Span::new(813, 822),
        scalar_field_name: Some("authorId1".to_string()),
        field_name: "author".to_string(),
        model_name: "Post".to_string(),
      },
      Error::RelationReferencedFieldNotFound {
        span: Span::new(978, 985),
        field_name: "negativeAuthor".to_string(),
        model_name: "Post".to_string(),
        referenced_field_name: "userId1".to_string(),
        referenced_model_name: "User".to_string(),
      },
      Error::RelationPartial {
        span: Span::new(215, 231),
        relation_name: "negative_posts".to_string(),
        field_name: None,
        model_name: None,
      },
    ];

    let decls = crate::parser::delcarations()
      .parse(&relation_errs_sdml)
      .into_result()
      .unwrap();

    match semantic_update(decls) {
      Ok(_) => assert!(false, "Expecting relation errors to surface"),
      Err(errs) => {
        eprintln!("{errs:#?}");
        assert_eq!(expected_semantic_errs.len(), errs.len());
        errs
          .into_iter()
          .for_each(|e| assert!(expected_semantic_errs.contains(&e)));
      }
    }
  }

  #[test]
  fn test_relation_errs_field_n_references() {
    let relation_errs_sdml = std::fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/semantic_analysis/relation_errs/field_n_references.sdml"
    ))
    .unwrap();
    let expected_semantic_errs: Vec<Error> = vec![
      Error::RelationReferencedFieldNotUnique {
        span: Span::new(1198, 1203),
        field_name: "user".to_string(),
        model_name: "Profile".to_string(),
        referenced_field_name: "email".to_string(),
        referenced_model_name: "User".to_string(),
      },
      Error::RelationScalarAndReferencedFieldsTypeMismatch {
        span: Span::new(586, 602),
        field_name: "spouseUserId".to_string(),
        model_name: "User".to_string(),
        referenced_field_name: "userId".to_string(),
        referenced_model_name: "User".to_string(),
      },
      Error::RelationScalarAndReferencedFieldsTypeMismatch {
        span: Span::new(724, 741),
        field_name: "authorId".to_string(),
        model_name: "Post".to_string(),
        referenced_field_name: "userId".to_string(),
        referenced_model_name: "User".to_string(),
      },
      Error::RelationScalarFieldIsNotPrimitive {
        span: Span::new(854, 871),
        field_name: "negativeAuthorId".to_string(),
        model_name: "Post".to_string(),
      },
      Error::RelationPartial {
        span: Span::new(142, 154),
        relation_name: "user_posts".to_string(),
        field_name: None,
        model_name: None,
      },
      Error::RelationPartial {
        span: Span::new(203, 219),
        relation_name: "negative_posts".to_string(),
        field_name: None,
        model_name: None,
      },
      Error::RelationPartial {
        span: Span::new(463, 477),
        relation_name: "user_profile".to_string(),
        field_name: None,
        model_name: None,
      },
    ];

    let decls = crate::parser::delcarations()
      .parse(&relation_errs_sdml)
      .into_result()
      .unwrap();
    match semantic_update(decls) {
      Ok(_) => assert!(false, "Expecting relation errors to surface"),
      Err(errs) => {
        assert_eq!(expected_semantic_errs.len(), errs.len());
        errs
          .into_iter()
          .for_each(|e| assert!(expected_semantic_errs.contains(&e)));
      }
    }
  }

  #[test]
  fn test_indexed_attribute_valid_usage() {
    let indexed_attribute_valid_usage_sdml = std::fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/semantic_analysis/indexed_attribute/indexed_attribute_valid_usage.sdml"
    ))
    .unwrap();

    let decls = crate::parser::delcarations()
      .parse(&indexed_attribute_valid_usage_sdml)
      .into_result()
      .unwrap();
    match semantic_update(decls) {
      Ok(_) => assert!(true, "test passed!"),
      Err(errs) => {
        assert!(
          false,
          "There shouldn't any scemantic errors instead {:?} thrown",
          errs
        )
      }
    }
  }

  #[test]
  fn test_indexed_attribute_invalid_usage() {
    let indexed_attribute_invalid_usage_sdml = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/semantic_analysis/indexed_attribute/indexed_attribute_invalid_usage.sdml"
        ))
        .unwrap();
    let expected_scemantic_errs: Vec<Error> = vec![
      Error::AttributeIncompatible {
        span: Span::new(148, 156),
        attrib_name: "indexed".to_string(),
        first_attrib_name: "unique".to_string(),
        field_name: "email".to_string(),
        model_name: "User".to_string(),
      },
      Error::AttributeIncompatible {
        span: Span::new(766, 774),
        attrib_name: "indexed".to_string(),
        first_attrib_name: "id".to_string(),
        field_name: "postId".to_string(),
        model_name: "Post".to_string(),
      },
      Error::RelationInvalidAttribute {
        span: Span::new(946, 954),
        attrib_name: "indexed".to_string(),
        field_name: "author".to_string(),
        model_name: "Post".to_string(),
      },
      Error::AttributeIncompatible {
        span: Span::new(946, 954),
        attrib_name: "indexed".to_string(),
        first_attrib_name: "relation".to_string(),
        field_name: "author".to_string(),
        model_name: "Post".to_string(),
      },
      Error::RelationPartial {
        span: Span::new(204, 216),
        relation_name: "user_posts".to_string(),
        field_name: None,
        model_name: None,
      },
    ];

    let decls = crate::parser::delcarations()
      .parse(&indexed_attribute_invalid_usage_sdml)
      .into_result()
      .unwrap();
    match semantic_update(decls) {
      Ok(_) => assert!(false, "Expecting semantic errors to get surfaced."),
      Err(errs) => {
        eprintln!("{:#?}", errs);
        errs.iter().for_each(|err| {
          assert!(
            expected_scemantic_errs.contains(err),
            "{} is an inexpected semantic error",
            err
          )
        });
      }
    }
  }
}
