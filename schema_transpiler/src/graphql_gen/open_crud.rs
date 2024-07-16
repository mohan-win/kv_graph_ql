//! Defines necessary abstractions to capture (OpenCRUD)[https://github.com/opencrud/opencrud]
//! types and interfaces in rust.
//!
//! **Terminology**
//!
//! Each application domain entity is represented as a model in SDML. And each model,
//! will have auto-generated CRUD interface comforming to OpenCRUD.
//! - Instance of a model entity is called object.
//!

/// Identifies various input types in OpenCRUD interface.
#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    Create(CreateInputType),
    Update(UpdateInputType),
    Filter(FilterType),
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
    UserConnectInput,
}

/// Identifies the input types used in filters
#[derive(Debug, Clone, PartialEq)]
pub enum FilterType {
    /// Identifies where critera where it can match one or more objects.
    WhereInput,
    /// Idenifies the where critrial where it can match at most one object.
    WhereUniqueInput,
}
