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
    Query(QueryType),
    Mutation(MutationType),
    Create(CreateInputArg),
    Update(UpdateInputArg),
    ConnectPos(ConnectPositionInputArg),
}

impl FieldNamedUnformatted for Field {
    fn name_str(&self, model_name: &str) -> String {
        match self {
            Self::Query(query) => query.name_str(model_name),
            Self::Mutation(mutation) => mutation.name_str(model_name),
            Self::Create(create_input) => create_input.name_str(model_name),
            Self::Update(update_input) => update_input.name_str(model_name),
            Self::ConnectPos(connect_pos_input) => connect_pos_input.name_str(model_name),
            _ => panic!("These are common fields, doesn't belong to a model."),
        }
    }
    fn common_name_str(&self) -> String {
        match self {
            Self::Id => "id".to_string(),
            Self::Query(query) => query.common_name_str(),
            Self::Mutation(mutation) => mutation.common_name_str(),
            Self::Create(create_input_field) => create_input_field.common_name_str(),
            Self::Update(update_input_field) => update_input_field.common_name_str(),
            Self::ConnectPos(connect_pos_input_field) => {
                connect_pos_input_field.common_name_str()
            }
            _ => panic!("These fields needs to be used in-context of a model."),
        }
    }
}

/// Fields for root Query type.
#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    // Root node field.
    RootNodeField,
    /// Root query field.
    RootField,
    /// Root query field array.
    RootFieldArray,
    /// Root Connection field.
    RootFieldConnection,
    // Query input arg,
    InputArg(QueryInputArg),
}

impl FieldNamedUnformatted for QueryType {
    fn name_str(&self, model_name: &str) -> String {
        match self {
            QueryType::RootNodeField => {
                panic!("Root node field is common for all models")
            }
            QueryType::RootField => model_name.to_string(),
            QueryType::RootFieldArray => pluralizer::pluralize(model_name, 2, false),
            QueryType::RootFieldConnection => {
                format!("{}Connection", pluralizer::pluralize(model_name, 2, false))
            }
            QueryType::InputArg(query_input_arg) => query_input_arg.name_str(model_name),
        }
    }
    fn common_name_str(&self) -> String {
        match self {
            QueryType::RootNodeField => "node".to_string(),
            QueryType::InputArg(query_input_arg) => query_input_arg.common_name_str(),
            fld => panic!("{:?} should be used in-context of a model.", fld),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryInputArg {
    Where,
    OrderBy,
    Skip,
    After,
    First,
    Before,
    Last,
}

impl FieldNamedUnformatted for QueryInputArg {
    fn name_str(&self, _model_name: &str) -> String {
        panic!("Input arg {:?} is not specific to model.", self)
    }
    fn common_name_str(&self) -> String {
        match self {
            Self::Where => "where",
            Self::OrderBy => "orderBy",
            Self::Skip => "skip",
            Self::After => "after",
            Self::First => "first",
            Self::Before => "before",
            Self::Last => "last",
        }
        .to_string()
    }
}

/// Fields for root mutation type.
#[derive(Debug, Clone, PartialEq)]
pub enum MutationType {
    Create,
    Update,
    Delete,
    Upsert,
    UpdateMany,
    DeleteMany,
    InputArg(MutationInputArg),
}

impl FieldNamedUnformatted for MutationType {
    fn name_str(&self, model_name: &str) -> String {
        let model_name_plural = pluralizer::pluralize(model_name, 2, false);
        match self {
            // Note: Intentionally inserted a "_" in the name, which is
            // removed when it is converted to camel_case in FieldNamed trait
            // function. Why? Thus allowing the name to be properly camelcased
            // even when model_name is mentioned without propercasing.
            Self::Create => format!("create_{model_name}"),
            Self::Update => format!("update_{model_name}"),
            Self::Delete => format!("delete_{model_name}"),
            Self::Upsert => format!("upsert_{model_name}"),
            Self::UpdateMany => format!("updateMany_{model_name_plural}Connection"),
            Self::DeleteMany => format!("deleteMany_{model_name_plural}Connection"),
            Self::InputArg(mutation_input_arg) => mutation_input_arg.name_str(model_name),
        }
    }
    fn common_name_str(&self) -> String {
        match self {
            Self::InputArg(mutation_input_arg) => mutation_input_arg.common_name_str(),
            _ => panic!("{:?} field should be used in-context of a model", self),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MutationInputArg {
    Where,
    Data,
    Skip,
    After,
    First,
    Before,
    Last,
}

impl FieldNamedUnformatted for MutationInputArg {
    fn name_str(&self, _model_name: &str) -> String {
        panic!("Input arg {:?} is not specific to model.", self)
    }
    fn common_name_str(&self) -> String {
        match self {
            Self::Where => "where",
            Self::Data => "data",
            Self::Skip => "skip",
            Self::After => "after",
            Self::First => "first",
            Self::Before => "before",
            Self::Last => "last",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreateInputArg {
    Create,
    Connect,
}

impl FieldNamedUnformatted for CreateInputArg {
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
pub enum UpdateInputArg {
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

impl FieldNamedUnformatted for UpdateInputArg {
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
pub enum ConnectPositionInputArg {
    // Identifies the field to connect after the speficied ID
    After,
    // Identifies the field to connect before the specified ID
    Before,
    // Identifies the field to connect at the first position
    Start,
    // Identifies the field to connect at the last position. [default]
    End,
}

impl FieldNamedUnformatted for ConnectPositionInputArg {
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
