//! Defines necessary abstractions to capture (OpenCRUD)[https://github.com/opencrud/opencrud]
//! field **names** in rust.

/// Trait exposing name of the OpenCRUD field.
pub trait FieldNamed {
    /// For the given model name return OpenCRUD field name.
    /// ## Arguments
    /// * `model_name` - name of the sdml model.
    fn named(&self, model_name: &str) -> String;
    /// Return the openCRUD field name.
    fn common_name(&self) -> String {
        panic!("This is a model (object.) specific field!")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Field {
    Id,
    Query(QueryField),
}

impl FieldNamed for Field {
    fn named(&self, model_name: &str) -> String {
        match self {
            Self::Query(query_fld) => query_fld.named(model_name),
            _ => panic!("These are common fields, doesn't belong to a model."),
        }
    }
    fn common_name(&self) -> String {
        match self {
            Self::Id => "id".to_string(),
            _ => panic!("These fields needs to be used in-context of a model."),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryField {
    RootNode,
    Connection,
}

impl FieldNamed for QueryField {
    fn named(&self, model_name: &str) -> String {
        match self {
            QueryField::RootNode => panic!("Root node field is common for all models"),
            QueryField::Connection => {
                let model_name_plural_lc = pluralizer::pluralize(model_name, 2, false);
                format!("{}Connection", model_name_plural_lc)
            }
        }
    }
    fn common_name(&self) -> String {
        match self {
            QueryField::RootNode => "node".to_string(),
            QueryField::Connection => {
                panic!("Connection should be used in-context of a model.")
            }
        }
    }
}
