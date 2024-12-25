use std::collections::HashMap;

use crate::{
  parser::semantic_analysis::{
    attribute::{
      ATTRIB_NAMED_ARG_FIELD, ATTRIB_NAMED_ARG_NAME, ATTRIB_NAMED_ARG_REFERENCES,
      ATTRIB_NAME_RELATION,
    },
    err::Error,
    visitor::VisitorMode,
    RelationMap, ATTRIB_NAME_ID, ATTRIB_NAME_UNIQUE,
  },
  types::{
    AttribArg, DeclarationsGrouped, FieldDecl, ModelDecl, NamedArg, RelationEdge, Token,
    Type,
  },
};

use super::*;

/// Visitor to update the all the model fielde of type Type::Unknown(..).
/// This visitor also takes care of capturing all the relations in the declarations.
#[derive(Debug, Default)]
pub struct UpdateUnknownFields {
  relation_map: Option<RelationMap>,
}

impl<'a> Visitor<'a> for UpdateUnknownFields {
  fn exit_field(&mut self, ctx: &mut VisitorContext<'a>, _field: &'a FieldDecl) {
    let _ = self
      .get_actual_type(ctx)
      .and_then(|field_type| {
        field_type.map(|field_type| {
          // Make sure to update the actual field_type on the updated_data_model.
          ctx.update_current_field_type(field_type);
        });
        Ok(())
      })
      .map_err(|err| {
        ctx.report_error(err);
      });
  }

  fn enter_declarations(
    &mut self,
    ctx: &mut VisitorContext<'a>,
    _declarations: &'a DeclarationsGrouped,
  ) {
    assert!(
      matches!(ctx.mode(), VisitorMode::Build(_)),
      "This visitor is valid only on `Build` mode."
    );

    self.relation_map = Some(RelationMap::default());
  }

  fn exit_declarations(
    &mut self,
    ctx: &mut VisitorContext<'a>,
    _declarations: &'a DeclarationsGrouped,
  ) {
    // Note: Updating all known relations to the `updated_data_model`.
    ctx.update_relation_map(self.relation_map.take().unwrap());
  }
}

impl UpdateUnknownFields {
  /// This function finds and returns the actual type for the field if it's type is Unknown
  /// If the field type is already known, it returns `None`.
  /// But if its unable to locate the Uknown type, then it returns SemanticError::UndefinedType.
  fn get_actual_type<'a>(
    &mut self,
    ctx: &mut VisitorContext<'a>,
  ) -> Result<Option<Type>, Error> {
    if let Type::Unknown(type_name_tok) = ctx.current_field.unwrap().field_type.r#type() {
      let type_name = type_name_tok.ident_name().unwrap();
      if ctx.current_field_relation.is_some() {
        // Is this a relation field ?
        let relation_edge = Self::get_relation_edge(
          ctx.current_field.unwrap(),
          ctx.current_model.unwrap(),
          ctx.current_field_relation.unwrap(),
        )?;
        // Make sure to add the relation_edge to the relation map.
        self.relation_map.as_mut().unwrap().add_relation_edge(
          relation_edge.clone(),
          &ctx.current_field.unwrap().name,
          &ctx.current_model.unwrap().name,
        )?;
        Ok(Some(Type::Relation(relation_edge)))
      } else if let Some(_) = ctx.input_enums().get(&type_name) {
        Ok(Some(Type::Enum {
          enum_ty_name: type_name_tok.clone(),
        }))
      } else {
        Err(Error::TypeUndefined {
          span: type_name_tok.span(),
          type_name,
          field_name: ctx.current_field.unwrap().name.ident_name().unwrap(),
          model_name: ctx.current_model.unwrap().name.ident_name().unwrap(),
        })
      }
    } else {
      Ok(None)
    }
  }

  pub fn get_relation_edge(
    field: &FieldDecl,
    model: &ModelDecl,
    referenced_model: &ModelDecl,
  ) -> Result<RelationEdge, Error> {
    let mut relation_attributes = Vec::new();
    let mut non_relation_attributes = Vec::new();
    field
      .attributes
      .iter()
      .for_each(|attrib| match &attrib.name {
        Token::Ident(name, _) if name == ATTRIB_NAME_RELATION => {
          relation_attributes.push(attrib)
        }
        _ => non_relation_attributes.push(attrib),
      });
    if relation_attributes.len() == 0 {
      // Throw error if there is no relation attribute.
      Err(Error::RelationAttributeMissing {
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
      Err(Error::RelationInvalidAttribute {
        span: invalid_attrib.name.span(),
        attrib_name: invalid_attrib.name.ident_name().unwrap(),
        field_name: field.name.ident_name().unwrap(),
        model_name: model.name.ident_name().unwrap(),
      })
    } else {
      let relation_attribute = relation_attributes[0];
      if let Some(AttribArg::Args(named_args)) = &relation_attribute.arg {
        Self::new_relation_edge(named_args, field, model, referenced_model)
      } else {
        Err(Error::RelationInvalidAttributeArg {
          span: relation_attribute.name.span(),
          relation_name: None,
          arg_name: None,
          field_name: field.name.ident_name(),
          model_name: model.name.ident_name(),
        })
      }
    }
  }

  fn new_relation_edge(
    relation_args: &Vec<NamedArg>,
    field: &FieldDecl,
    model: &ModelDecl,
    referenced_model: &ModelDecl,
  ) -> Result<RelationEdge, Error> {
    let RelationAttributeDetails {
      relation_name,
      relation_scalar_field,
      referenced_model_field,
      referenced_model_relation_field,
    } = Self::get_relation_attribute_args(relation_args, field, model, referenced_model)?;

    if relation_scalar_field.is_none() && referenced_model_field.is_none() {
      // Is this OneSideRelation ?
      return Ok(RelationEdge::OneSideRelation {
        relation_name: relation_name.clone(),
        referenced_model_name: referenced_model.name.clone(),
      });
    } else if relation_scalar_field.is_none() {
      return Err(Error::RelationScalarFieldNotFound {
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
      .find(|attrib| matches!(&attrib.name, Token::Ident(ident_name, _) if ident_name == ATTRIB_NAME_UNIQUE))
      .is_some();
    let rel_fld_exists = referenced_model_relation_field.is_some();
    let rel_fld_array =
      referenced_model_relation_field.is_some_and(|fld| fld.field_type.is_array());

    match (scalar_fld_unique, rel_fld_exists, rel_fld_array) {
      (_scalar_fld_unique @ true, _rel_fld_exists @ true, _rel_fld_array @ true) => {
        Err(Error::RelationScalarFieldIsUnique {
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
        Err(Error::RelationScalarFieldNotUnique {
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
        Err(Error::RelationScalarFieldNotUnique {
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
      _ => Err(Error::RelationInvalid {
        span: relation_name.span(),
        relation_name: relation_name.str().unwrap(),
        field_name: field.name.ident_name(),
        model_name: model.name.ident_name(),
      }),
    }
  }

  fn get_relation_attribute_args<'b>(
    relation_args: &'b Vec<NamedArg>,
    field: &'b FieldDecl,
    model: &'b ModelDecl,
    referenced_model: &'b ModelDecl,
  ) -> Result<RelationAttributeDetails<'b>, Error> {
    let mut relation_name: Option<&'b Token> = None;
    let mut relation_scalar_field: Option<&'b FieldDecl> = None;
    let mut referenced_model_field: Option<&'b FieldDecl> = None;
    let referenced_model_relation_field: Option<&'b FieldDecl>;

    // Step 1: Validate relation attribute has correct set of args.
    let mut valid_arg_sets: HashMap<usize, Vec<_>> = HashMap::new();
    valid_arg_sets.insert(1, vec![ATTRIB_NAMED_ARG_NAME]);
    valid_arg_sets.insert(
      3,
      vec![
        ATTRIB_NAMED_ARG_NAME,
        ATTRIB_NAMED_ARG_FIELD,
        ATTRIB_NAMED_ARG_REFERENCES,
      ],
    );
    // Check for invalid arg sets
    let allowed_arg_set = valid_arg_sets.get(&relation_args.len());
    if allowed_arg_set.is_none() {
      return Err(Error::RelationInvalidAttributeArg {
        span: field.name.span(),
        relation_name: None,
        arg_name: None,
        field_name: field.name.ident_name(),
        model_name: model.name.ident_name(),
      });
    } else {
      relation_args.iter().try_for_each(|arg| {
        if allowed_arg_set
          .unwrap()
          .contains(&arg.arg_name.ident_name().unwrap().as_str())
        {
          Ok(())
        } else {
          Err(Error::RelationInvalidAttributeArg {
            span: field.name.span(),
            relation_name: None,
            arg_name: arg.arg_name.ident_name(),
            field_name: field.name.ident_name(),
            model_name: model.name.ident_name(),
          })
        }
      })?;
    }

    // Step 2: Get those arg values, and make sure they are of expected type.
    for arg in relation_args.iter() {
      match &arg.arg_name {
        Token::Ident(ident_name, _) if ident_name == ATTRIB_NAMED_ARG_NAME => {
          if let Token::String(..) = arg.arg_value {
            relation_name = Some(&arg.arg_value);
          } else {
            return Err(Error::RelationInvalidAttributeArg {
              span: arg.arg_value.span(),
              relation_name: None,
              arg_name: arg.arg_name.ident_name(),
              field_name: field.name.ident_name(),
              model_name: model.name.ident_name(),
            });
          }
        }
        Token::Ident(ident_name, _) if ident_name == ATTRIB_NAMED_ARG_FIELD => {
          relation_scalar_field =
            Some(Self::get_relation_scalar_field(arg, field, model)?);
        }
        Token::Ident(ident_name, _) if ident_name == ATTRIB_NAMED_ARG_REFERENCES => {
          referenced_model_field = Some(Self::get_referenced_model_field(
            arg,
            field,
            model,
            referenced_model,
          )?);
        }
        _ => {
          return Err(Error::RelationInvalidAttributeArg {
            span: arg.arg_value.span(),
            relation_name: None,
            arg_name: arg.arg_name.ident_name(),
            field_name: field.name.ident_name(),
            model_name: model.name.ident_name(),
          })
        }
      }
    }

    referenced_model_relation_field = Self::get_referenced_model_relation_field(
      relation_name.expect("relation_name can't be None at this point."),
      field,
      model,
      referenced_model,
    );

    // Make sure relation scalar field and referenced field are of the `same primitive type`
    if relation_scalar_field.is_some()
      && referenced_model_field.is_some()
      && *relation_scalar_field.unwrap().field_type.r#type()
        != *referenced_model_field.unwrap().field_type.r#type()
    {
      Err(Error::RelationScalarAndReferencedFieldsTypeMismatch {
        span: relation_scalar_field.unwrap().name.span(),
        field_name: relation_scalar_field.unwrap().name.ident_name().unwrap(),
        model_name: model.name.ident_name().unwrap(),
        referenced_field_name: referenced_model_field.unwrap().name.ident_name().unwrap(),
        referenced_model_name: referenced_model.name.ident_name().unwrap(),
      })
    } else {
      Ok(RelationAttributeDetails {
        relation_name: relation_name.expect("relation_name can't be None at this point."),
        relation_scalar_field,
        referenced_model_field,
        referenced_model_relation_field,
      })
    }
  }

  fn get_relation_scalar_field<'src, 'b>(
    relation_arg_field: &'b NamedArg,
    field: &'b FieldDecl,
    model: &'b ModelDecl,
  ) -> Result<&'b FieldDecl, Error> {
    debug_assert!(
      matches!(&relation_arg_field.arg_name, Token::Ident(ident_name, _) if ident_name == ATTRIB_NAMED_ARG_FIELD),
      "Invalid argument passed for relation_arg_field"
    );

    let relation_scalar_field: Option<&'b FieldDecl>;
    // Validate relation scalar field.
    // Make sure relation scalar field exists in the parent model.
    if let Token::Ident(..) = relation_arg_field.arg_value {
      relation_scalar_field = model
        .fields
        .iter()
        .find(|field| field.name == relation_arg_field.arg_value);
      if relation_scalar_field.is_none() {
        Err(Error::RelationScalarFieldNotFound {
          span: relation_arg_field.arg_value.span(),
          scalar_field_name: relation_arg_field.arg_value.ident_name(),
          field_name: field.name.ident_name().unwrap(),
          model_name: model.name.ident_name().unwrap(),
        })
      } else if relation_scalar_field.is_some_and(|relation_scalar_field| {
        // If relation_scalar_field is not a scalar, then throw error.
        match &*relation_scalar_field.field_type.r#type() {
          Type::Primitive { .. } => false,
          _ => true,
        }
      }) {
        Err(Error::RelationScalarFieldIsNotPrimitive {
          span: relation_scalar_field.unwrap().name.span(),
          field_name: relation_scalar_field.unwrap().name.ident_name().unwrap(),
          model_name: model.name.ident_name().unwrap(),
        })
      } else {
        Ok(relation_scalar_field.unwrap())
      }
    } else {
      Err(Error::RelationInvalidAttributeArg {
        span: relation_arg_field.arg_value.span(),
        relation_name: None,
        arg_name: relation_arg_field.arg_name.ident_name(),
        field_name: Some(field.name.ident_name().unwrap()),
        model_name: Some(model.name.ident_name().unwrap()),
      })
    }
  }

  fn get_referenced_model_field<'src, 'b>(
    relation_arg_references: &'b NamedArg,
    field: &'b FieldDecl,
    model: &'b ModelDecl,
    referenced_model: &'b ModelDecl,
  ) -> Result<&'b FieldDecl, Error> {
    debug_assert!(
      matches!(&relation_arg_references.arg_name, Token::Ident(ident_name, _) if ident_name == ATTRIB_NAMED_ARG_REFERENCES),
      "Invalid arg is passed for relation_arg_references"
    );
    let referenced_model_field: Option<&'b FieldDecl>;
    // Validate referenced field.
    // Make sure referenced field, exists in the referenced model.
    if let Token::Ident(_, _) = relation_arg_references.arg_value {
      referenced_model_field = referenced_model
        .fields
        .iter()
        .find(|field| relation_arg_references.arg_value == field.name);
      if referenced_model_field.is_none() {
        Err(Error::RelationReferencedFieldNotFound {
          span: relation_arg_references.arg_value.span(),
          field_name: field.name.ident_name().unwrap(),
          model_name: model.name.ident_name().unwrap(),
          referenced_field_name: relation_arg_references.arg_value.ident_name().unwrap(),
          referenced_model_name: referenced_model.name.ident_name().unwrap(),
        })
      } else if referenced_model_field.is_some_and(|referenced_model_field| {
        match &*referenced_model_field.field_type.r#type() {
          Type::Primitive { .. } => false,
          _ => true,
        }
      }) {
        // If referenced field is not scalar throw an error
        Err(Error::RelationReferencedFieldNotScalar {
          span: relation_arg_references.arg_value.span(),
          field_name: field.name.ident_name().unwrap(),
          model_name: model.name.ident_name().unwrap(),
          referenced_field_name: referenced_model_field
            .unwrap()
            .name
            .ident_name()
            .unwrap(),
          referenced_model_name: referenced_model.name.ident_name().unwrap(),
        })
      } else if referenced_model_field.is_some_and(|referenced_model_field| {
        referenced_model_field
          .attributes
          .iter()
          .find(|attrib| match &attrib.name {
            Token::Ident(ident_name, _) if ident_name == ATTRIB_NAME_UNIQUE => true,
            Token::Ident(ident_name, _) if ident_name == ATTRIB_NAME_ID => true,
            _ => false,
          })
          .is_none()
      }) {
        // if the referenced field is not attributed with @id or @unique then throw error.
        Err(Error::RelationReferencedFieldNotUnique {
          span: relation_arg_references.arg_value.span(),
          field_name: field.name.ident_name().unwrap(),
          model_name: model.name.ident_name().unwrap(),
          referenced_field_name: referenced_model_field
            .unwrap()
            .name
            .ident_name()
            .unwrap(),
          referenced_model_name: referenced_model.name.ident_name().unwrap(),
        })
      } else {
        Ok(referenced_model_field.unwrap())
      }
    } else {
      Err(Error::RelationInvalidAttributeArg {
        span: relation_arg_references.arg_value.span(),
        relation_name: None,
        arg_name: relation_arg_references.arg_name.ident_name(),
        field_name: field.name.ident_name(),
        model_name: model.name.ident_name(),
      })
    }
  }

  fn get_referenced_model_relation_field<'src, 'b>(
    relation_name: &'b Token,
    field: &'b FieldDecl,
    model: &'b ModelDecl,
    referenced_model: &'b ModelDecl,
  ) -> Option<&'b FieldDecl> {
    let is_self_relation = model.name == referenced_model.name;
    let mut referenced_model_relation_field =
      referenced_model.fields.iter().filter(|fld| {
        // Does this field has relation attribute to it ?
        let mut has_relation_attribute = fld.attributes.iter().filter(|attrib| {
          match &attrib.name {
            Token::Ident(name, _) if name == ATTRIB_NAME_RELATION => {
              // In case of self relation, make sure to pick the right relation field.
              true && (!is_self_relation || fld.name != field.name)
            }
            _ => false,
          }
        });

        if let Some(relation_attrib) = has_relation_attribute.next() {
          // if yes, then does field type & relation name matches ?
          match &relation_attrib.arg {
            Some(AttribArg::Args(named_args)) => {
              named_args.iter().fold(false, |acc, named_arg| {
                if relation_name == &named_arg.arg_value
                  && fld.field_type.r#type().token() == &model.name
                {
                  acc || true
                } else {
                  acc || false
                }
              })
            }
            _ => false,
          }
        } else {
          false
        }
      });

    referenced_model_relation_field.next()
  }
}

struct RelationAttributeDetails<'a> {
  pub relation_name: &'a Token,
  pub relation_scalar_field: Option<&'a FieldDecl>,
  pub referenced_model_field: Option<&'a FieldDecl>,
  pub referenced_model_relation_field: Option<&'a FieldDecl>,
}
