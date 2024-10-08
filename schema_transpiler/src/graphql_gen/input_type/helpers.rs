use super::*;

#[derive(Debug, Clone)]
pub(super) struct ModelFields<'src, 'b> {
    pub non_relation_fields: Vec<&'b sdml_ast::FieldDecl<'src>>,
    pub relation_fields: Vec<&'b sdml_ast::FieldDecl<'src>>,
}
/// Get Model fields.
/// ### Arguments
/// * `model` - reference to sdml model declaration
/// * `filter_relation_scalar_fields` - if `true`, the function filters out the relation scalar fields.
pub(super) fn get_model_fields<'src, 'b>(
    model: &'b sdml_ast::ModelDecl<'src>,
    filter_relation_scalar_fields: bool,
    filter_auto_generated_id: bool,
) -> ModelFields<'src, 'b> {
    let mut non_relation_fields = vec![];
    let mut relation_fields = vec![];
    let mut relation_scalar_field_names = vec![];
    model
        .fields
        .iter()
        .for_each(|field| match &*field.field_type.r#type() {
            sdml_ast::Type::Unknown(..) => {
                panic!("Can't transpile field with unknown type to graphql")
            }
            sdml_ast::Type::Relation(edge) => {
                relation_fields.push(field);
                edge.scalar_field_name().map(|fld_name| {
                    relation_scalar_field_names.push(fld_name.ident_name())
                });
            }
            sdml_ast::Type::Primitive { .. } | sdml_ast::Type::Enum { .. } => {
                non_relation_fields.push(field);
            }
        });

    if filter_relation_scalar_fields || filter_auto_generated_id {
        non_relation_fields = non_relation_fields
            .into_iter()
            .filter(|field| {
                (!filter_relation_scalar_fields
                    || !relation_scalar_field_names.contains(&field.name.ident_name()))
                    && (!filter_auto_generated_id || !field.is_auto_gen_id())
            })
            .collect();
    }

    ModelFields {
        non_relation_fields,
        relation_fields,
    }
}
