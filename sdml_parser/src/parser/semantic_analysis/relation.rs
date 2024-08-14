use std::collections::HashMap;

use crate::ast::{AttribArg, FieldDecl, ModelDecl, NamedArg, RelationEdge, Token, Type};

use super::{
    attribute::{
        ATTRIB_NAMED_ARG_FIELD, ATTRIB_NAMED_ARG_NAME, ATTRIB_NAMED_ARG_REFERENCES, ATTRIB_NAME_ID,
        ATTRIB_NAME_RELATION, ATTRIB_NAME_UNIQUE,
    },
    err::SemanticError,
};

#[derive(Debug)]
pub(crate) struct RelationMap<'src, 'rel>
where
    'rel: 'src,
{
    relations: HashMap<
        &'src str,
        (
            Option<&'rel RelationEdge<'src>>,
            Option<&'rel RelationEdge<'src>>,
        ),
    >,
}

impl<'src, 'rel> RelationMap<'src, 'rel> {
    pub fn new() -> Self {
        RelationMap {
            relations: HashMap::new(),
        }
    }

    /// Returns valid relations with fully formed edges.
    /// If partially formed relations or invalid relations are present,
    /// then error is returned.
    pub fn get_valid_relations(
        &'src self,
    ) -> Result<
        HashMap<&'src str, (&'rel RelationEdge<'src>, &'rel RelationEdge<'src>)>,
        Vec<SemanticError<'src>>,
    > {
        let mut valid_relations = HashMap::new();
        let mut errs = Vec::new();
        self.relations.iter().for_each(|(key, (left, right))| {
            RelationMap::is_relation_valid(left.as_deref(), right.as_deref()).map_or_else(
                |e| errs.push(e),
                |_| {
                    valid_relations.insert(*key, (left.unwrap(), right.unwrap()));
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
        edge: &'rel RelationEdge<'src>,
        parent_field_ident: &Token<'src>,
        parent_model_ident: &Token<'src>,
    ) -> Result<(), SemanticError<'src>> {
        let relation_name = edge.relation_name();
        let relation_name_str = relation_name.ident_name().unwrap();
        let updated_relation = self.relations.get_mut(relation_name_str).map_or_else(
            || match edge {
                RelationEdge::OneSideRelationRight { .. }
                | RelationEdge::ManySideRelation { .. } => Ok((None, Some(edge))),

                _ => Ok((Some(edge), None)),
            },
            |existing_relation| match existing_relation {
                (None, Some(right)) => Ok((Some(edge), Some(right))),
                (Some(left), None) => Ok((Some(left), Some(edge))),
                (Some(_), Some(_)) => Err(SemanticError::RelationInvalid {
                    span: relation_name.span(),
                    relation_name: relation_name_str,
                    field_name: parent_field_ident.ident_name().unwrap(),
                    model_name: parent_model_ident.ident_name().unwrap(),
                }),
                (None, None) => panic!("This can't happen"),
            },
        )?;
        self.relations.insert(relation_name_str, updated_relation);
        Ok(())
    }

    /// Checks if the relation edges are valid and the relation is fully formed
    /// with the given 2 edges.
    fn is_relation_valid(
        left: Option<&'rel RelationEdge<'src>>,
        right: Option<&'rel RelationEdge<'src>>,
    ) -> Result<(), SemanticError<'src>> {
        match (left, right) {
            (None, None) => {
                panic!("Empty relations are not allowed!")
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
    let mut relation_name = None;
    let mut relation_scalar_field = None;
    let mut referenced_model_field = None;
    for arg in relation_args.iter() {
        let invalid_arg_err = Err(SemanticError::RelationInvalidAttributeArg {
            span: arg.arg_value.span(),
            field_name: Some(field.name.ident_name().unwrap()),
            model_name: Some(model.name.ident_name().unwrap()),
        });
        match arg.arg_name {
            Token::Ident(ATTRIB_NAMED_ARG_NAME, _) => {
                if let Token::Str(..) = arg.arg_value {
                    relation_name = Some(arg.arg_value.clone())
                } else {
                    return invalid_arg_err;
                }
            }
            Token::Ident(ATTRIB_NAMED_ARG_FIELD, _) => {
                // Make sure relation scalar field exists in the parent model.
                if let Token::Ident(..) = arg.arg_value {
                    relation_scalar_field = model
                        .fields
                        .iter()
                        .find(|field| field.name == arg.arg_value);
                    if relation_scalar_field.is_none() {
                        return Err(SemanticError::RelationScalarFieldNotFound {
                            span: arg.arg_value.span(),
                            field_name: field.name.ident_name().unwrap(),
                            model_name: model.name.ident_name().unwrap(),
                        });
                    } else if relation_scalar_field.is_some_and(|relation_scalar_field| {
                        // If relation_scalar_field is not a scalar, then throw error.
                        match &*relation_scalar_field.field_type.r#type() {
                            Type::Primitive { .. } => false,
                            _ => true,
                        }
                    }) {
                        return Err(SemanticError::RelationScalarFieldIsNotScalar {
                            span: relation_scalar_field.unwrap().name.span(),
                            field_name: relation_scalar_field.unwrap().name.ident_name().unwrap(),
                            model_name: model.name.ident_name().unwrap(),
                        });
                    }
                } else {
                    return invalid_arg_err;
                }
            }
            Token::Ident(ATTRIB_NAMED_ARG_REFERENCES, _) => {
                // Make sure referenced field, exists in the referenced model.
                if let Token::Ident(_, _) = arg.arg_value {
                    referenced_model_field = referenced_model
                        .fields
                        .iter()
                        .find(|field| arg.arg_value == field.name);
                    if referenced_model_field.is_none() {
                        return Err(SemanticError::RelationReferencedFieldNotFound {
                            span: arg.arg_value.span(),
                            field_name: field.name.ident_name().unwrap(),
                            model_name: model.name.ident_name().unwrap(),
                            referenced_field_name: arg.arg_value.ident_name().unwrap(),
                            referenced_model_name: referenced_model.name.ident_name().unwrap(),
                        });
                    } else if referenced_model_field.is_some_and(|referenced_model_field| {
                        match &*referenced_model_field.field_type.r#type() {
                            Type::Primitive { .. } => false,
                            _ => true,
                        }
                    }) {
                        // If referenced field is not scalar throw an error
                        return Err(SemanticError::RelationReferencedFieldNotScalar {
                            span: arg.arg_value.span(),
                            field_name: field.name.ident_name().unwrap(),
                            model_name: model.name.ident_name().unwrap(),
                            referenced_field_name: referenced_model_field
                                .unwrap()
                                .name
                                .ident_name()
                                .unwrap(),
                            referenced_model_name: referenced_model.name.ident_name().unwrap(),
                        });
                    } else if referenced_model_field.is_some_and(|referenced_model_field| {
                        referenced_model_field
                            .attributes
                            .iter()
                            .find(|attrib| match attrib.name {
                                Token::Ident(ATTRIB_NAME_UNIQUE, ..)
                                | Token::Ident(ATTRIB_NAME_ID, ..) => true,
                                _ => false,
                            })
                            .is_none()
                    }) {
                        // if the referenced field is not attributed with @id or @unique then throw error.
                        return Err(SemanticError::RelationReferencedFieldNotUnique {
                            span: arg.arg_value.span(),
                            field_name: field.name.ident_name().unwrap(),
                            model_name: model.name.ident_name().unwrap(),
                            referenced_field_name: referenced_model_field
                                .unwrap()
                                .name
                                .ident_name()
                                .unwrap(),
                            referenced_model_name: referenced_model.name.ident_name().unwrap(),
                        });
                    }
                } else {
                    return invalid_arg_err;
                }
            }
            _ => return invalid_arg_err,
        }
    }

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

    // Is this OneSideRelationRight ?
    if relation_scalar_field.is_some()
        && referenced_model_field
            .is_some_and(|referenced_model_field| !referenced_model_field.field_type.is_array)
    {
        return Ok(RelationEdge::OneSideRelationRight {
            relation_name: relation_name.clone(),
            scalar_field_name: relation_scalar_field.unwrap().name.clone(),
            referenced_model_name: referenced_model.name.clone(),
            referenced_field_name: referenced_model_field.unwrap().name.clone(),
        });
    }

    // Is this ManySideRelation ?
    if relation_scalar_field.is_some()
        && referenced_model_field
            .is_some_and(|referenced_model_field| referenced_model_field.field_type.is_array)
    {
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
