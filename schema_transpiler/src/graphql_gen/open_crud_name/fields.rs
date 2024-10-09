//! Defines necessary abstractions to capture (OpenCRUD)[https://github.com/opencrud/opencrud]
//! field **names** in rust.

use convert_case::{self, Casing};
use graphql_value::Name;

/// Trait exposing name of the OpenCRUD field.
/// **Note** Do not implement this trait, instead implement *FieldNamedUnformatted* trait.
pub trait FieldNamed {
    /// For the given model name return OpenCRUD field name.
    /// ### Arguments
    /// * `model_name` - name of the sdml model.
    fn name(&self, model_name: &str) -> Name;
    /// Return the openCRUD field name.
    fn common_name(&self) -> Name {
        panic!("This is a model (object.) specific field!")
    }
}

impl<F> FieldNamed for F
where
    F: FieldNamedUnformatted,
{
    fn name(&self, model_name: &str) -> Name {
        Name::new(self.name_str(model_name).to_case(convert_case::Case::Camel))
    }
    fn common_name(&self) -> Name {
        Name::new(self.common_name_str().to_case(convert_case::Case::Camel))
    }
}

/// Trait exposing the name of the OpenCRUD field.
/// **Note:**
/// * This is a private trait.
/// * Properly formatted field names are exposed via
///   *FieldNamed* which is automatically implemented for for field types
///   which implementes this trait.
trait FieldNamedUnformatted {
    fn name_str(&self, model_name: &str) -> String;
    fn common_name_str(&self) -> String;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Field {
    Id,
    Query(QueryField),
    Create(CreateInputField),
    Update(UpdateInputField),
    ConnectPos(ConnectPositionInputField),
}

impl FieldNamedUnformatted for Field {
    fn name_str(&self, model_name: &str) -> String {
        match self {
            Self::Query(query_fld) => query_fld.name_str(model_name),
            Self::Create(create_input_field) => create_input_field.name_str(model_name),
            Self::Update(update_input_field) => update_input_field.name_str(model_name),
            Self::ConnectPos(connect_pos_input_field) => {
                connect_pos_input_field.name_str(model_name)
            }
            _ => panic!("These are common fields, doesn't belong to a model."),
        }
    }
    fn common_name_str(&self) -> String {
        match self {
            Self::Id => "id".to_string(),
            Self::Create(create_input_field) => create_input_field.common_name_str(),
            Self::Update(update_input_field) => update_input_field.common_name_str(),
            Self::ConnectPos(connect_pos_input_field) => {
                connect_pos_input_field.common_name_str()
            }
            _ => panic!("These fields needs to be used in-context of a model."),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryField {
    RootNode,
    /// Root query field.
    RootField,
    /// Root query field array.
    RootFieldArray,
    /// Root Connection field.
    RootFieldConnection,
}

impl FieldNamedUnformatted for QueryField {
    fn name_str(&self, model_name: &str) -> String {
        match self {
            QueryField::RootNode => panic!("Root node field is common for all models"),
            QueryField::RootField => model_name.to_string(),
            QueryField::RootFieldArray => pluralizer::pluralize(model_name, 2, false),
            QueryField::RootFieldConnection => {
                format!("{}Connection", pluralizer::pluralize(model_name, 2, false))
            }
        }
    }
    fn common_name_str(&self) -> String {
        match self {
            QueryField::RootNode => "node".to_string(),
            fld => panic!("{:?} should be used in-context of a model.", fld),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreateInputField {
    Create,
    Connect,
}

impl FieldNamedUnformatted for CreateInputField {
    fn name_str(&self, _model_name: &str) -> String {
        match self {
            fld => panic!("{:?} common for all the model. Doesn't changes its name based on model name.", fld),
        }
    }

    fn common_name_str(&self) -> String {
        match self {
            Self::Create => "create".to_string(),
            Self::Connect => "connect".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateInputField {
    Create,
    Connect,
    Disconnect,
    Set,
    Update,
    Upsert,
    Delete,
    Where,
    Data,
    ConnectPosition,
}

impl FieldNamedUnformatted for UpdateInputField {
    fn name_str(&self, _model_name: &str) -> String {
        match self {
            fld => panic!("{:?} common for all the model. Doesn't changes its name based on model name.", fld),
        }
    }

    fn common_name_str(&self) -> String {
        match self {
            Self::Create => "create".to_string(),
            Self::Connect => "connect".to_string(),
            Self::Disconnect => "disconnect".to_string(),
            Self::Set => "set".to_string(),
            Self::Update => "update".to_string(),
            Self::Upsert => "upsert".to_string(),
            Self::Delete => "delete".to_string(),
            Self::Where => "where".to_string(),
            Self::Data => "data".to_string(),
            Self::ConnectPosition => "position".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectPositionInputField {
    // Identifies the field to connect after the speficied ID
    After,
    // Identifies the field to connect before the specified ID
    Before,
    // Identifies the field to connect at the first position
    Start,
    // Identifies the field to connect at the last position. [default]
    End,
}

impl FieldNamedUnformatted for ConnectPositionInputField {
    fn name_str(&self, _model_name: &str) -> String {
        match self {
            fld => panic!("{:?} common for all the model. Doesn't changes its name based on model name.", fld),
        }
    }
    fn common_name_str(&self) -> String {
        match self {
            Self::After => "after".to_string(),
            Self::Before => "before".to_string(),
            Self::Start => "start".to_string(),
            Self::End => "end".to_string(),
        }
    }
}
