use std::collections::HashMap;

use crate::{
  parser::semantic_analysis::{
    attribute::{
      ATTRIB_NAMED_ARG_FIELD, ATTRIB_NAMED_ARG_NAME, ATTRIB_NAMED_ARG_REFERENCES,
      ATTRIB_NAME_RELATION,
    },
    RelationMap, ATTRIB_NAME_ID, ATTRIB_NAME_UNIQUE,
  },
  types::{NamedArg, RelationEdge, Token, Type},
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
    self.get_actual_type(ctx).map(|field_type| {
      // Make sure to update the actual field_type on the updated_data_model.
      ctx.update_current_field_type(field_type);
    });
  }

  fn enter_declarations(
    &mut self,
    _ctx: &mut VisitorContext<'a>,
    _declarations: &'a DeclarationsGrouped,
  ) {
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
  fn get_actual_type<'a>(&mut self, ctx: &mut VisitorContext<'a>) -> Option<Type> {
    if let Type::Unknown(type_name_tok) = ctx.current_field.unwrap().field_type.r#type() {
      let type_name = type_name_tok.ident_name().unwrap();
      if ctx.current_field_relation.is_some() {
        // Is this a relation field ?
        return Self::get_relation_edge(
          ctx.current_field.unwrap(),
          ctx.current_model.unwrap(),
          ctx.current_field_relation.unwrap(),
        )
        .map(|relation_edge| {
          // Make sure to add the relation_edge to the relation map.
          let _ = self
            .relation_map
            .as_mut()
            .unwrap()
            .add_relation_edge(
              relation_edge.clone(),
              &ctx.current_field.unwrap().name,
              &ctx.current_model.unwrap().name,
            )
            .map_err(|err| {
              ctx.report_error(err);
            });
          Type::Relation(relation_edge)
        });
      } else if let Some(_) = ctx.input_enums().get(&type_name) {
        return Some(Type::Enum {
          enum_ty_name: type_name_tok.clone(),
        });
      }
    }

    None
  }

  fn get_relation_edge<'a>(
    field: &'a FieldDecl,
    model: &'a ModelDecl,
    referenced_model: &'a ModelDecl,
  ) -> Option<RelationEdge> {
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
    // Note: No need to do validation here.
    // It can be done in validate_field_attribute, validate_field_attribute_relation modules.
    if relation_attributes.len() == 1 {
      if let Some(AttribArg::Args(named_args)) = &relation_attributes[0].arg {
        return Self::new_relation_edge(named_args, field, model, referenced_model);
      }
    }

    None
  }

  fn new_relation_edge<'a>(
    relation_args: &Vec<NamedArg>,
    field: &'a FieldDecl,
    model: &'a ModelDecl,
    referenced_model: &'a ModelDecl,
  ) -> Option<RelationEdge> {
    let RelationAttributeDetails {
      relation_name,
      relation_scalar_field,
      referenced_model_field,
      referenced_model_relation_field,
    } = Self::get_relation_attribute_args(relation_args, field, model, referenced_model)?;

    if relation_scalar_field.is_none() && referenced_model_field.is_none() {
      // Is this OneSideRelation ?
      return Some(RelationEdge::OneSideRelation {
        relation_name: relation_name.clone(),
        referenced_model_name: referenced_model.name.clone(),
      });
    } else if relation_scalar_field.is_none() {
      return None;
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
        // Error::RelationScalarFieldIsUnique
        None
      }
      (_scalar_fld_unique @ false, _rel_fld_exists @ true, _rel_fld_array @ false) => {
        // Error::RelationScalarFieldNotUnique
        None
      }
      (_scalar_fld_unique @ false, _rel_fld_exists @ false, _rel_fld_array @ _)
        if model.name == referenced_model.name =>
      {
        // Self relation should always be a 1-to-1 relation.
        // Error::RelationScalarFieldNotUnique
        None
      }
      (_scalar_fld_unique @ true, _rel_fld_exists @ false, _)
        if model.name == referenced_model.name =>
      {
        // See if this is a self relation
        Some(RelationEdge::SelfOneToOneRelation {
          relation_name: relation_name.clone(),
          scalar_field_name: relation_scalar_field.name.clone(),
          referenced_model_name: referenced_model.name.clone(),
          referenced_field_name: referenced_model_field.unwrap().name.clone(),
        })
      }
      (_scalar_fld_unique @ true, _rel_fld_exists @ true, _rel_fld_array @ false) => {
        Some(RelationEdge::OneSideRelationRight {
          relation_name: relation_name.clone(),
          scalar_field_name: relation_scalar_field.name.clone(),
          referenced_model_name: referenced_model.name.clone(),
          referenced_field_name: referenced_model_field.unwrap().name.clone(),
        })
      }
      (_scalar_fld_unique @ false, _rel_fld_exists @ true, _rel_fld_array @ true) => {
        Some(RelationEdge::ManySideRelation {
          relation_name: relation_name.clone(),
          scalar_field_name: relation_scalar_field.name.clone(),
          referenced_model_name: referenced_model.name.clone(),
          referenced_field_name: referenced_model_field.unwrap().name.clone(),
        })
      }
      _ => {
        // Error::RelationInvalid
        None
      }
    }
  }

  fn get_relation_attribute_args<'a>(
    relation_args: &'a Vec<NamedArg>,
    field: &'a FieldDecl,
    model: &'a ModelDecl,
    referenced_model: &'a ModelDecl,
  ) -> Option<RelationAttributeDetails<'a>> {
    let mut relation_name: Option<&'a Token> = None;
    let mut relation_scalar_field: Option<&'a FieldDecl> = None;
    let mut referenced_model_field: Option<&'a FieldDecl> = None;
    let referenced_model_relation_field: Option<&'a FieldDecl>;

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
    // Step 1: Make sure the `relation_args` in the allowed arg set. Otherwise, return `None`.
    let allowed_arg_set = valid_arg_sets.get(&relation_args.len())?;
    if relation_args.iter().fold(true, |acc, arg| {
      acc && allowed_arg_set.contains(&arg.arg_name.ident_name().unwrap().as_str())
    }) == false
    {
      return None;
    }

    // Step 2: Get those arg values, and make sure they are of expected type.
    relation_args
      .iter()
      .try_for_each(|arg| match &arg.arg_name {
        Token::Ident(ident_name, _) if ident_name == ATTRIB_NAMED_ARG_NAME => {
          if let Token::String(..) = arg.arg_value {
            relation_name = Some(&arg.arg_value);
            Ok(())
          } else {
            Err(())
          }
        }
        Token::Ident(ident_name, _) if ident_name == ATTRIB_NAMED_ARG_FIELD => {
          relation_scalar_field =
            Some(Self::get_relation_scalar_field(arg, model).ok_or(())?);
          Ok(())
        }
        Token::Ident(ident_name, _) if ident_name == ATTRIB_NAMED_ARG_REFERENCES => {
          referenced_model_field =
            Some(Self::get_referenced_model_field(arg, referenced_model).ok_or(())?);
          Ok(())
        }
        _ => Err(()),
      })
      .ok()?;

    referenced_model_relation_field = Some(Self::get_referenced_model_relation_field(
      relation_name.unwrap().str().unwrap(),
      field,
      model,
      referenced_model,
    )?);

    Some(RelationAttributeDetails {
      relation_name: relation_name.unwrap(),
      referenced_model_field,
      relation_scalar_field,
      referenced_model_relation_field,
    })
  }

  fn get_relation_scalar_field<'a>(
    relation_arg_field: &'a NamedArg,
    model: &'a ModelDecl,
  ) -> Option<&'a FieldDecl> {
    debug_assert!(
      matches!(&relation_arg_field.arg_name, Token::Ident(ident_name, _) if ident_name == ATTRIB_NAMED_ARG_FIELD),
      "Invalid argument passed for relation_arg_field"
    );

    let relation_scalar_field: Option<&'a FieldDecl>;
    // Validate relation scalar field.
    // Make sure relation scalar field exists in the parent model.
    if let Token::Ident(..) = relation_arg_field.arg_value {
      relation_scalar_field = model
        .fields
        .iter()
        .find(|field| field.name == relation_arg_field.arg_value);

      if relation_scalar_field.is_some_and(|relation_scalar_field| {
        // If relation_scalar_field should be scalar!
        match &*relation_scalar_field.field_type.r#type() {
          Type::Primitive { .. } => true,
          _ => false,
        }
      }) {
        return relation_scalar_field;
      }
    }
    None
  }

  fn get_referenced_model_field<'a>(
    relation_arg_references: &'a NamedArg,
    referenced_model: &'a ModelDecl,
  ) -> Option<&'a FieldDecl> {
    debug_assert!(
      matches!(&relation_arg_references.arg_name, Token::Ident(ident_name, _) if ident_name == ATTRIB_NAMED_ARG_REFERENCES),
      "Invalid arg is passed for relation_arg_references"
    );
    let referenced_model_field: Option<&'a FieldDecl>;
    // Validate referenced field.
    // Make sure referenced field, exists in the referenced model.
    if let Token::Ident(_, _) = relation_arg_references.arg_value {
      referenced_model_field = referenced_model
        .fields
        .iter()
        .find(|field| relation_arg_references.arg_value == field.name);

      // Make sure the referenced model field is of scalar type.
      if referenced_model_field.is_some_and(|referenced_model_field| {
        match &*referenced_model_field.field_type.r#type() {
          Type::Primitive { .. } => true,
          _ => false,
        }
      }) && referenced_model_field.is_some_and(|referenced_model_field| {
        // Make sure referenced field is attributed with @id or @unique.
        referenced_model_field
          .attributes
          .iter()
          .find(|attrib| match &attrib.name {
            Token::Ident(ident_name, _) if ident_name == ATTRIB_NAME_UNIQUE => true,
            Token::Ident(ident_name, _) if ident_name == ATTRIB_NAME_ID => true,
            _ => false,
          })
          .is_some()
      }) {
        return referenced_model_field;
      }
    }
    None
  }

  fn get_referenced_model_relation_field<'a>(
    relation_name: String,
    field: &'a FieldDecl,
    model: &'a ModelDecl,
    referenced_model: &'a ModelDecl,
  ) -> Option<&'a FieldDecl> {
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
              named_args
                .iter()
                .fold(false, |acc, named_arg| match &named_arg.arg_value {
                  Token::Ident(rel_name, _)
                    if rel_name.eq(&relation_name)
                      && fld.field_type.r#type().token() == &model.name =>
                  {
                    acc || true
                  }
                  _ => acc || false,
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
