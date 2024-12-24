use std::collections::HashMap;

use crate::types::{RelationEdge, Token};

use super::err::Error;

#[derive(Debug, Default)]
pub(crate) struct RelationMap {
  relations: HashMap<String, (Option<RelationEdge>, Option<RelationEdge>)>,
}

impl RelationMap {
  /// Returns valid relations with fully formed edges.
  /// If partially formed relations or invalid relations are present,
  /// then error is returned.
  pub fn get_valid_relations(
    self,
  ) -> Result<HashMap<String, (RelationEdge, Option<RelationEdge>)>, Vec<Error>> {
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

  #[must_use]
  pub fn add_relation_edge(
    &mut self,
    edge: RelationEdge,
    parent_field_ident: &Token,
    parent_model_ident: &Token,
  ) -> Result<(), Error> {
    let relation_name_str = edge.relation_name().str().unwrap();
    if let Some(existing_relation) = self.relations.get_mut(&relation_name_str) {
      match existing_relation {
        (Some(_), Some(_)) | (Some(RelationEdge::SelfOneToOneRelation { .. }), None) => {
          Err(Error::RelationDuplicate {
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
    left: Option<&RelationEdge>,
    right: Option<&RelationEdge>,
  ) -> Result<(), Error> {
    match (left, right) {
      (None, None) => {
        panic!("Empty relations are not allowed!")
      }
      (Some(RelationEdge::SelfOneToOneRelation { .. }), None) => {
        // For self one-to-one relation, we only need one edge.
        Ok(())
      }
      (Some(..), None) => Err(Error::RelationPartial {
        span: left.unwrap().relation_name().span(),
        relation_name: left.unwrap().relation_name().str().unwrap(),
        field_name: None,
        model_name: None,
      }),
      (None, Some(..)) => Err(Error::RelationPartial {
        span: right.unwrap().relation_name().span(),
        relation_name: right.unwrap().relation_name().str().unwrap(),
        field_name: None,
        model_name: None,
      }),
      (
        Some(RelationEdge::OneSideRelation { .. }),
        Some(RelationEdge::OneSideRelation { .. }),
      ) => Err(Error::RelationInvalid {
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
      ) => Err(Error::RelationInvalid {
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
