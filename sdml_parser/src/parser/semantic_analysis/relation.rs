use std::collections::HashMap;

use crate::ast::{AttribArg, FieldDecl, ModelDecl, NamedArg, RelationEdge, Token, Type};

use super::{
    attribute::{
        validate_relation_attribute_args, RelationAttributeDetails, ATTRIB_NAMED_ARG_FIELD,
        ATTRIB_NAMED_ARG_NAME, ATTRIB_NAMED_ARG_REFERENCES, ATTRIB_NAME_ID, ATTRIB_NAME_RELATION,
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
        let relation_name = edge.relation_name();
        let relation_name_str = relation_name.ident_name().unwrap();
        if let Some(existing_relation) = self.relations.get_mut(relation_name_str) {
            match existing_relation {
                (Some(_), Some(_)) | (Some(RelationEdge::SelfOneToOneRelation { .. }), None) => {
                    Err(SemanticError::RelationDuplicate {
                        span: relation_name.span(),
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
                relation_name: left.unwrap().relation_name().ident_name().unwrap(),
                field_name: None,
                model_name: None,
            }),
            (None, Some(..)) => Err(SemanticError::RelationPartial {
                span: right.unwrap().relation_name().span(),
                relation_name: right.unwrap().relation_name().ident_name().unwrap(),
                field_name: None,
                model_name: None,
            }),
            (
                Some(RelationEdge::OneSideRelation { .. }),
                Some(RelationEdge::OneSideRelation { .. }),
            ) => Err(SemanticError::RelationInvalidAttributeArg {
                span: right.unwrap().relation_name().span(),
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
            ) => Err(SemanticError::RelationInvalidAttributeArg {
                span: left.unwrap().relation_name().span(),
                field_name: None,
                model_name: None,
            }),
            (Some(..), Some(..)) => {
                assert!(
                    left.unwrap().relation_name() == right.unwrap().relation_name(),
                    "Relation name should match for the edges in a relation"
                );
                eprintln!("Valid relations left:{:#?} right:{:#?}", left, right); // ToDo:: remove!
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
        Err(SemanticError::RelationNoAttribute {
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
                field_name: Some(field.name.ident_name().unwrap()),
                model_name: Some(model.name.ident_name().unwrap()),
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
    } = validate_relation_attribute_args(relation_args, field, model, referenced_model)?;

    if relation_name.is_none() {
        // Name is mandatory for relation attribute, if not present throw error.
        return Err(SemanticError::RelationAttributeMissingName {
            span: field.name.span(),
            field_name: field.name.ident_name().unwrap(),
            model_name: model.name.ident_name().unwrap(),
        });
    }
    let relation_name = relation_name.unwrap();

    // Is this OneSideRelation ?
    if relation_scalar_field.is_none() && referenced_model_field.is_none() {
        return Ok(RelationEdge::OneSideRelation {
            relation_name: relation_name.clone(),
            referenced_model_name: referenced_model.name.clone(),
        });
    }

    let relation_scalar_field_is_unique =
        relation_scalar_field.is_some_and(|relation_scalar_field| {
            relation_scalar_field
                .attributes
                .iter()
                .find(|attrib| match attrib.name {
                    Token::Ident(ATTRIB_NAME_UNIQUE, ..) => true,
                    _ => false,
                })
                .is_some()
        });

    // Is this OneSideRelationRight ?
    if relation_scalar_field_is_unique {
        // See if this is a self relation
        if model.name == referenced_model.name {
            return Ok(RelationEdge::SelfOneToOneRelation {
                relation_name: relation_name.clone(),
                scalar_field_name: relation_scalar_field.unwrap().name.clone(),
                referenced_model_name: referenced_model.name.clone(),
                referenced_field_name: referenced_model_field.unwrap().name.clone(),
            });
        } else {
            return Ok(RelationEdge::OneSideRelationRight {
                relation_name: relation_name.clone(),
                scalar_field_name: relation_scalar_field.unwrap().name.clone(),
                referenced_model_name: referenced_model.name.clone(),
                referenced_field_name: referenced_model_field.unwrap().name.clone(),
            });
        }
    } else if referenced_model_field.is_some() && !relation_scalar_field_is_unique {
        // Is this ManySideRelation ?
        return Ok(RelationEdge::ManySideRelation {
            relation_name: relation_name.clone(),
            scalar_field_name: relation_scalar_field.unwrap().name.clone(),
            referenced_model_name: referenced_model.name.clone(),
            referenced_field_name: referenced_model_field.unwrap().name.clone(),
        });
    }

    Err(SemanticError::RelationInvalid {
        span: relation_name.span(),
        relation_name: relation_name.ident_name().unwrap(),
        field_name: field.name.ident_name().unwrap(),
        model_name: model.name.ident_name().unwrap(),
    })
}
