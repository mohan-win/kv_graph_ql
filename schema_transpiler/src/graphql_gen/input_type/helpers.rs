use super::*;

#[derive(Debug, Clone)]
pub(super) struct ModelFields<'src, 'b> {
    pub relation_fields: Vec<&'b sdml_ast::FieldDecl<'src>>,
    pub relation_scalar_fields: Vec<&'b sdml_ast::FieldDecl<'src>>,
    pub id_fields: Vec<(&'b sdml_ast::FieldDecl<'src>, bool)>, // Vec<(field, is_auto_generated)>
    pub unique_fields: Vec<&'b sdml_ast::FieldDecl<'src>>,
    pub non_unique_fields: Vec<&'b sdml_ast::FieldDecl<'src>>,
}

/// Get Model fields.
pub(super) fn get_model_fields<'src, 'b>(
    model: &'b sdml_ast::ModelDecl<'src>,
) -> ModelFields<'src, 'b> {
    let mut result = ModelFields {
        relation_fields: Vec::new(),
        relation_scalar_fields: Vec::new(),
        id_fields: Vec::new(),
        unique_fields: Vec::new(),
        non_unique_fields: Vec::new(),
    };

    let mut relation_scalar_field_names = Vec::new();
    model
        .fields
        .iter()
        .for_each(|field| match &*field.field_type.r#type() {
            sdml_ast::Type::Unknown(..) => {
                panic!("Can't transpile field with unknown type to graphql")
            }
            sdml_ast::Type::Relation(edge) => {
                result.relation_fields.push(field);
                edge.scalar_field_name().map(|fld_name| {
                    relation_scalar_field_names.push(fld_name.ident_name().unwrap())
                });
            }
            sdml_ast::Type::Primitive { .. } | sdml_ast::Type::Enum { .. } => {
                if field.is_auto_gen_id() {
                    result.id_fields.push((field, true));
                } else if field.has_id_attrib() {
                    result.id_fields.push((field, false));
                } else if field.has_unique_attrib() {
                    result.unique_fields.push(field);
                } else {
                    result.non_unique_fields.push(field);
                }
            }
        });

    let mut relation_scalar_fields = Vec::new();
    // Filter-out relation scalar fields from unique & non-unique fields.
    result.unique_fields = result
        .unique_fields
        .into_iter()
        .filter(|field| {
            if relation_scalar_field_names.contains(&field.name.ident_name().unwrap()) {
                relation_scalar_fields.push(*field);
                false
            } else {
                true
            }
        })
        .collect();
    result.non_unique_fields = result
        .non_unique_fields
        .into_iter()
        .filter(|field| {
            if relation_scalar_field_names.contains(&field.name.ident_name().unwrap()) {
                relation_scalar_fields.push(*field);
                false
            } else {
                true
            }
        })
        .collect();

    result
}
