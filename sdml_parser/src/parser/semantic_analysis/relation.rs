use std::collections::HashMap;

use crate::ast::{AttribArg, FieldDecl, ModelDecl, NamedArg, RelationEdge, Token};

use super::{
  attribute::{
    validate_relation_attribute_args, RelationAttributeDetails, ATTRIB_NAME_RELATION,
    ATTRIB_NAME_UNIQUE,
  },
  err::SemanticError,
};

#[derive(Debug)]
pub(crate) struct RelationMap<'src> {
  relations: HashMap<&'src str, (Option<RelationEdge<'src>>, Option<RelationEdge<'src>>)>,
}

impl<'src> RelationMap<'src> {
  pub fn new() -> Self {
    RelationMap {
      relations: HashMap::new(),
    }
  }

  /// Returns valid relations with fully formed edges.
  /// If partially formed relations or invalid relations are present,
  /// then error is returned.
  pub fn get_valid_relations(
    self,
  ) -> Result<
    HashMap<&'src str, (RelationEdge<'src>, Option<RelationEdge<'src>>)>,
    Vec<SemanticError<'src>>,
  > {
    let mut valid_relations = HashMap::new();
    let mut errs = Vec::new();
    self.relations.into_iter().for_each(|(key, (left, right))| {
      RelationMap::is_relation_valid(left.as_ref(), right.as_ref()).map_or_else(
        |e| errs.push(e),
        |_| {
          valid_relations.insert(key, (left.unwrap(), right));
        },
      );
    });
    if errs.len() > 0 {
      Err(errs)
    } else {
      Ok(valid_relations)
    }
  }

  pub fn add_relation_edge(
    &mut self,
    edge: RelationEdge<'src>,
    parent_field_ident: &Token<'src>,
    parent_model_ident: &Token<'src>,
  ) -> Result<(), SemanticError<'src>> {
    let relation_name_str = edge.relation_name().str().unwrap();
    if let Some(existing_relation) = self.relations.get_mut(relation_name_str) {
      match existing_relation {
        (Some(_), Some(_)) | (Some(RelationEdge::SelfOneToOneRelation { .. }), None) => {
          Err(SemanticError::RelationDuplicate {
            span: edge.relation_name().span(),
            relation_name: relation_name_str,
            field_name: parent_field_ident.ident_name().unwrap(),
            model_name: parent_model_ident.ident_name().unwrap(),
          })
        }

        (None, Some(_)) => {
          existing_relation.0 = Some(edge);
          Ok(())
        }
        (Some(_), None) => {
          existing_relation.1 = Some(edge);
          Ok(())
        }
        (None, None) => panic!("This can't happen"),
      }?;
    } else {
      let new_relation = match edge {
        RelationEdge::OneSideRelationRight { .. }
        | RelationEdge::ManySideRelation { .. } => (None, Some(edge)),

        _ => (Some(edge), None),
      };
      self.relations.insert(relation_name_str, new_relation);
    }
    Ok(())
  }

  /// Checks if the relation edges are valid and the relation is fully formed
  /// with the given 2 edges.
  fn is_relation_valid(
    left: Option<&RelationEdge<'src>>,
    right: Option<&RelationEdge<'src>>,
  ) -> Result<(), SemanticError<'src>> {
    match (left, right) {
      (None, None) => {
        panic!("Empty relations are not allowed!")
      }
      (Some(RelationEdge::SelfOneToOneRelation { .. }), None) => {
        // For self one-to-one relation, we only need one edge.
        Ok(())
      }
      (Some(..), None) => Err(SemanticError::RelationPartial {
        span: left.unwrap().relation_name().span(),
        relation_name: left.unwrap().relation_name().str().unwrap(),
        field_name: None,
        model_name: None,
      }),
      (None, Some(..)) => Err(SemanticError::RelationPartial {
        span: right.unwrap().relation_name().span(),
        relation_name: right.unwrap().relation_name().str().unwrap(),
        field_name: None,
        model_name: None,
      }),
      (
        Some(RelationEdge::OneSideRelation { .. }),
        Some(RelationEdge::OneSideRelation { .. }),
      ) => Err(SemanticError::RelationInvalid {
        span: right.unwrap().relation_name().span(),
        relation_name: right.unwrap().relation_name().str().unwrap(),
        field_name: None,
        model_name: None,
      }),
      (Some(RelationEdge::OneSideRelationRight { .. }), _)
      | (
        Some(RelationEdge::ManySideRelation { .. }),
        Some(RelationEdge::OneSideRelationRight { .. }),
      )
      | (
        Some(RelationEdge::ManySideRelation { .. }),
        Some(RelationEdge::OneSideRelation { .. }),
      ) => Err(SemanticError::RelationInvalid {
        span: left.unwrap().relation_name().span(),
        relation_name: left.unwrap().relation_name().str().unwrap(),
        field_name: None,
        model_name: None,
      }),
      (Some(..), Some(..)) => {
        assert!(
          left.unwrap().relation_name() == right.unwrap().relation_name(),
          "Relation name should match for the edges in a relation"
        );
        Ok(())
      }
    }
  }
}

pub fn get_relation_edge<'src>(
  model: &ModelDecl<'src>,
  field: &FieldDecl<'src>,
  referenced_model: &ModelDecl<'src>,
) -> Result<RelationEdge<'src>, SemanticError<'src>> {
  let mut relation_attributes = Vec::new();
  let mut non_relation_attributes = Vec::new();
  field.attributes.iter().for_each(|attrib| {
    if let Token::Ident(ATTRIB_NAME_RELATION, _) = attrib.name {
      relation_attributes.push(attrib)
    } else {
      non_relation_attributes.push(attrib)
    }
  });
  if relation_attributes.len() == 0 {
    // Throw error if there is no relation attribute.
    Err(SemanticError::RelationAttributeMissing {
      span: field.name.span(),
      field_name: field.name.ident_name().unwrap(),
      model_name: model.name.ident_name().unwrap(),
    })
  } else if non_relation_attributes.len() > 0 || relation_attributes.len() > 1 {
    // Return error if there is a non-relation attribute or duplicate relation attributes
    let invalid_attrib = relation_attributes
      .get(1)
      .or(non_relation_attributes.get(0))
      .unwrap();
    Err(SemanticError::RelationInvalidAttribute {
      span: invalid_attrib.name.span(),
      attrib_name: invalid_attrib.name.ident_name().unwrap(),
      field_name: field.name.ident_name().unwrap(),
      model_name: model.name.ident_name().unwrap(),
    })
  } else {
    let relation_attribute = relation_attributes[0];
    if let Some(AttribArg::Args(named_args)) = &relation_attribute.arg {
      new_relation_edge(model, named_args, field, referenced_model)
    } else {
      Err(SemanticError::RelationInvalidAttributeArg {
        span: relation_attribute.name.span(),
        relation_name: None,
        arg_name: None,
        field_name: field.name.ident_name(),
        model_name: model.name.ident_name(),
      })
    }
  }
}

fn new_relation_edge<'src>(
  model: &ModelDecl<'src>,
  relation_args: &Vec<NamedArg<'src>>,
  field: &FieldDecl<'src>,
  referenced_model: &ModelDecl<'src>,
) -> Result<RelationEdge<'src>, SemanticError<'src>> {
  let RelationAttributeDetails {
    relation_name,
    relation_scalar_field,
    referenced_model_field,
    referenced_model_relation_field,
  } = validate_relation_attribute_args(relation_args, field, model, referenced_model)?;

  if relation_scalar_field.is_none() && referenced_model_field.is_none() {
    // Is this OneSideRelation ?
    return Ok(RelationEdge::OneSideRelation {
      relation_name: relation_name.clone(),
      referenced_model_name: referenced_model.name.clone(),
    });
  } else if relation_scalar_field.is_none() {
    return Err(SemanticError::RelationScalarFieldNotFound {
      span: relation_name.span(),
      scalar_field_name: None,
      field_name: field.name.ident_name().unwrap(),
      model_name: model.name.ident_name().unwrap(),
    });
  }
  let relation_scalar_field = relation_scalar_field.unwrap();

  let scalar_fld_unique = relation_scalar_field
    .attributes
    .iter()
    .find(|attrib| match attrib.name {
      Token::Ident(ATTRIB_NAME_UNIQUE, ..) => true,
      _ => false,
    })
    .is_some();
  let rel_fld_exists = referenced_model_relation_field.is_some();
  let rel_fld_array =
    referenced_model_relation_field.is_some_and(|fld| fld.field_type.is_array());

  match (scalar_fld_unique, rel_fld_exists, rel_fld_array) {
    (_scalar_fld_unique @ true, _rel_fld_exists @ true, _rel_fld_array @ true) => {
      Err(SemanticError::RelationScalarFieldIsUnique {
        span: relation_scalar_field.name.span(),
        field_name: relation_scalar_field.name.ident_name().unwrap(),
        model_name: model.name.ident_name().unwrap(),
        referenced_model_name: referenced_model.name.ident_name().unwrap(),
        referenced_model_relation_field_name: referenced_model_relation_field
          .unwrap()
          .name
          .ident_name()
          .unwrap(),
      })
    }
    (_scalar_fld_unique @ false, _rel_fld_exists @ true, _rel_fld_array @ false) => {
      Err(SemanticError::RelationScalarFieldNotUnique {
        span: relation_scalar_field.name.span(),
        field_name: relation_scalar_field.name.ident_name().unwrap(),
        model_name: model.name.ident_name().unwrap(),
        referenced_model_name: referenced_model.name.ident_name().unwrap(),
        referenced_model_relation_field_name: referenced_model_relation_field
          .map_or(None, |fld| fld.name.ident_name()),
      })
    }
    (_scalar_fld_unique @ false, _rel_fld_exists @ false, _rel_fld_array @ _)
      if model.name == referenced_model.name =>
    {
      // Self relation should always be a 1-to-1 relation.
      Err(SemanticError::RelationScalarFieldNotUnique {
        span: relation_scalar_field.name.span(),
        field_name: relation_scalar_field.name.ident_name().unwrap(),
        model_name: model.name.ident_name().unwrap(),
        referenced_model_name: referenced_model.name.ident_name().unwrap(),
        referenced_model_relation_field_name: referenced_model_relation_field
          .map_or(None, |fld| fld.name.ident_name()),
      })
    }
    (_scalar_fld_unique @ true, _rel_fld_exists @ false, _)
      if model.name == referenced_model.name =>
    {
      // See if this is a self relation
      Ok(RelationEdge::SelfOneToOneRelation {
        relation_name: relation_name.clone(),
        scalar_field_name: relation_scalar_field.name.clone(),
        referenced_model_name: referenced_model.name.clone(),
        referenced_field_name: referenced_model_field.unwrap().name.clone(),
      })
    }
    (_scalar_fld_unique @ true, _rel_fld_exists @ true, _rel_fld_array @ false) => {
      Ok(RelationEdge::OneSideRelationRight {
        relation_name: relation_name.clone(),
        scalar_field_name: relation_scalar_field.name.clone(),
        referenced_model_name: referenced_model.name.clone(),
        referenced_field_name: referenced_model_field.unwrap().name.clone(),
      })
    }
    (_scalar_fld_unique @ false, _rel_fld_exists @ true, _rel_fld_array @ true) => {
      Ok(RelationEdge::ManySideRelation {
        relation_name: relation_name.clone(),
        scalar_field_name: relation_scalar_field.name.clone(),
        referenced_model_name: referenced_model.name.clone(),
        referenced_field_name: referenced_model_field.unwrap().name.clone(),
      })
    }
    _ => Err(SemanticError::RelationInvalid {
      span: relation_name.span(),
      relation_name: relation_name.str().unwrap(),
      field_name: field.name.ident_name(),
      model_name: model.name.ident_name(),
    }),
  }
}
