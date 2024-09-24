//! Defines necessary abstractions to capture (OpenCRUD)[https://github.com/opencrud/opencrud]
//! types and interfaces in rust.
//!
//! **Terminology**
//!
//! Each application domain entity is represented as a model in SDML. And each model,
//! will have auto-generated CRUD interface comforming to OpenCRUD.
//! - Instance of a model entity is called object.

use super::Type;

/// Trait exposing the name of the OpenCRUD abstraction.
pub trait Named {
    /// For the given model name return OpenCRUD abstraction name(a.k.a identifier).
    /// # Arguments
    /// `model_name` - name of the sdml model.
    fn name(&self, model_name: &str) -> String;
    /// For the given model with name,
    /// return OpenCRUD abstraction identifier's GraphQL type.
    /// # Arguments
    /// `model_name` - name of the sdml model.
    /// `type_mod` - type modifier.
    fn ty(&self, model_name: &str, type_mod: sdml_parser::ast::FieldTypeMod) -> Type {
        Type::new(&self.name(model_name), type_mod)
    }
}

/// Identifies various input types in OpenCRUD interface.
#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    Create(CreateInputType),
    Update(UpdateInputType),
    Filter(FilterType),
    OrderBy,
}

impl Named for InputType {
    fn name(&self, model_name: &str) -> String {
        match self {
            InputType::Create(create_input_type) => create_input_type.name(model_name),
            InputType::Update(update_input_type) => update_input_type.name(model_name),
            InputType::Filter(filter_input_type) => filter_input_type.name(model_name),
            InputType::OrderBy => format!("{model_name}OrderByInput"),
        }
    }
}

/// Identifies auxiliary [output] types in OpenCRUD interface.
#[derive(Debug, Clone, PartialEq)]
pub enum AuxiliaryType {
    Edge,
    Connection,
}

impl Named for AuxiliaryType {
    fn name(&self, model_name: &str) -> String {
        match self {
            AuxiliaryType::Edge => format!("{model_name}Edge"),
            AuxiliaryType::Connection => format!("{model_name}Connection"),
        }
    }
}

/// Identifies input types used in create interfaces.
#[derive(Debug, Clone, PartialEq)]
pub enum CreateInputType {
    /// Identifies input type used to create a new object.
    /// Ex. UserCreateInput creates a new user.
    CreateInput,
    /// Identifies the input type used to create the many objects in a relation
    /// in a nested create.
    /// Ex. PostCreateManyInlineInput will be used inside UserCreateInput
    /// to create posts inline when creating a new user.
    CreateManyInlineInput,
    /// Identifies the input type used to create one object in one side of the relation
    /// in a nested create.
    /// Ex. ProfileCreateOneInlineInput will be used inside UserCreateInput
    /// to create user profile inline when creating a new user.
    CreateOneInlineInput,
}

impl Named for CreateInputType {
    fn name(&self, model_name: &str) -> String {
        match self {
            CreateInputType::CreateInput => format!("{model_name}CreateInput"),
            CreateInputType::CreateManyInlineInput => {
                format!("{model_name}CreateManyInlineInput")
            }
            CreateInputType::CreateOneInlineInput => {
                format!("{model_name}CreateOneInlineInput")
            }
        }
    }
}

/// Identifies input types used in update or upsert interfaces.
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateInputType {
    /// Identifies the input type use to update a object.
    /// Ex. UserUpdateInput is used to capture
    /// the *complete data* to update a single user object including contained relations.
    UpdateInput,
    /// Identifies the input type used to upsert a object.
    /// Ex. UserUpsertInput is used to capture the required data
    /// to update the *complete record* of a user including its contained relations,
    /// if the user exists, or to create the user if user doesn't exists.
    UpsertInput,
    /// Identifies the input type used to capture the data for many-side of the relation for updates.
    /// Ex. PostUpdateManyInlineInput[] is used inside UserUpdateInput to update many
    /// posts beloning to the user who is being updated.
    UpdateManyInlineInput,
    /// Identifies the input type used to capture the data for one-side of the relation for updates.
    /// Ex. ProfileUpdateOneInlineInput is used inside UserUpdateInput to update the
    /// profile blonging to the user who is being updated.
    UpdateOneInlineInput,
    /// Used inside UpdateManyInlineInput::update field to capture the updates to the many side of the relation
    /// where each update is accompanied with a unique where condition.
    /// Ex. UserUpdateWithNestedWhereUniqueInput is used inside UserUpdateManyInlineInput to update the
    /// user meeting the where unique condition when user is in a many side of the relationship.
    UpdateWithNestedWhereUniqueInput,
    /// Used inside UpdateManyInlineInput::upsert field to capture the upserts to the many side of the relation
    /// where each update is accompanied with a unique where condition.
    /// Ex. UserUpsertWithNestedWhereUniqueInput is used inside UserUpdateManyInlineInput to upsert the
    /// user meeting the where unique condition when user is in a many side of the relationship.
    UpsertWithNestedWhereUniqueInput,
    /// Identifies the input type specifying the existing object to connect to a relation.
    /// Ex. UserConnectInput is used inside UserUpdateManyInlineInput to connect existing users
    /// in a many side of relation.
    ConnectInput,
}

impl Named for UpdateInputType {
    fn name(&self, model_name: &str) -> String {
        match self {
            UpdateInputType::UpdateInput => format!("{model_name}UpdateInput"),
            UpdateInputType::UpsertInput => format!("{model_name}UpsertInput"),
            UpdateInputType::UpdateManyInlineInput => {
                format!("{model_name}UpdateManyInlineInput")
            }
            UpdateInputType::UpdateOneInlineInput => {
                format!("{model_name}UpdateOneInlineInput")
            }
            UpdateInputType::UpdateWithNestedWhereUniqueInput => {
                format!("{model_name}UpdateWithNestedWhereUniqueInput")
            }
            UpdateInputType::UpsertWithNestedWhereUniqueInput => {
                format!("{model_name}UpsertWithNestedWhereUniqueInput")
            }
            UpdateInputType::ConnectInput => {
                format!("{model_name}ConnectInput")
            }
        }
    }
}

/// Identifies the input types used in filters
#[derive(Debug, Clone, PartialEq)]
pub enum FilterType {
    /// Identifies where critera where it can match one or more objects.
    WhereInput,
    /// Idenifies the where critrial where it can match at most one object.
    WhereUniqueInput,
}

impl Named for FilterType {
    fn name(&self, model_name: &str) -> String {
        match self {
            FilterType::WhereInput => format!("{model_name}WhereInput"),
            FilterType::WhereUniqueInput => format!("{model_name}WhereUniqueInput"),
        }
    }
}
