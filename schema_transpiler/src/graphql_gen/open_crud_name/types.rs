//! Defines necessary abstractions to capture (OpenCRUD)[https://github.com/opencrud/opencrud]
//! types and interfaces **names** in rust.
//!
//! **Terminology**
//!
//! Each application domain entity is represented as a model in SDML. And each model,
//! will have auto-generated CRUD interface comforming to OpenCRUD.
//! - Instance of a model entity is called object.

use convert_case::Casing;

use super::{Name, Type, TypeMod};

/// Trait exposing the name & type of the OpenCRUD abstraction.
/// ** Note **
/// * Do not implement this trait for exposing OpenCRUDType name,
/// instead implement the trait *NamedUnformatted*
pub trait Named {
    /// For the given model name return OpenCRUD abstraction name(a.k.a identifier).
    /// ### Arguments
    /// * `model_name` - name of the model from SDML.
    fn name(&self, model_name: &str) -> Name;
    /// For the given model with name,
    /// return OpenCRUD abstraction identifier's GraphQL type.
    /// ### Arguments
    /// * `model_name` - name of the sdml model.
    /// * `type_mod` - type modifier.
    fn ty(&self, model_name: &str, type_mod: TypeMod) -> Type {
        Type::new(&self.name(model_name), type_mod)
    }
    /// Get *common* openCRUD abstraction name.
    fn common_name(&self) -> Name;
    /// Get *common* openCRUD abstraction type.
    fn common_ty(&self, type_mod: TypeMod) -> Type {
        Type::new(&self.common_name(), type_mod)
    }
}

impl<T> Named for T
where
    T: NamedUnformatted,
{
    fn name(&self, model_name: &str) -> Name {
        Name::new(self.name_str(&model_name.to_case(convert_case::Case::Pascal)))
    }
    fn common_name(&self) -> Name {
        Name::new(self.common_name_str())
    }
}

/// Trait exposing the name of the OpenCRUD abstraction.
/// ** Note **
/// * This is a private trait.
/// * Properly formatted OpenCRUD Type names are exposed via *Named* trait
/// which is automatically implemented for all OpenCRUDType(s) which implements
/// this trait.
trait NamedUnformatted {
    /// Return the name of the OpenCRUDType for the given
    /// `model_name_pc` (model name in *Pascal Case*).
    fn name_str(&self, model_name_pc: &str) -> String;
    fn common_name_str(&self) -> String {
        panic!("Common name for this abstraction is not available. This abstraction should be used in-conext of a specific model.");
    }
}

/// Identifies various input types in OpenCRUD interface.
#[derive(Debug, Clone, PartialEq)]
pub enum OpenCRUDType {
    IdType,
    Mutation(MutationType),
    Query(QueryType),
    Create(CreateInput),
    Update(UpdateInput),
    Filter(FilterInput),
    OrderByInput,
}

impl NamedUnformatted for OpenCRUDType {
    fn name_str(&self, model_name_pc: &str) -> String {
        match self {
            OpenCRUDType::IdType => panic!("ID type is not model specific."),
            OpenCRUDType::Query(query_type) => query_type.name_str(model_name_pc),
            OpenCRUDType::Mutation(mutation_type) => {
                mutation_type.name_str(model_name_pc)
            }
            OpenCRUDType::Create(create_input_type) => {
                create_input_type.name_str(model_name_pc)
            }
            OpenCRUDType::Update(update_input_type) => {
                update_input_type.name_str(model_name_pc)
            }
            OpenCRUDType::Filter(filter_input_type) => {
                filter_input_type.name_str(model_name_pc)
            }
            OpenCRUDType::OrderByInput => {
                format!("{model_name_pc}OrderByInput")
            }
        }
    }
    fn common_name_str(&self) -> String {
        match self {
            OpenCRUDType::IdType => "ID".to_string(),
            OpenCRUDType::Query(query_type) => query_type.common_name_str(),
            OpenCRUDType::Mutation(mutation_type) => mutation_type.common_name_str(),
            OpenCRUDType::Create(create_input_type) => {
                create_input_type.common_name_str()
            }
            OpenCRUDType::Update(update_input_type) => {
                update_input_type.common_name_str()
            }
            OpenCRUDType::Filter(filter_input_type) => {
                filter_input_type.common_name_str()
            }
            OpenCRUDType::OrderByInput => {
                panic!("OrderBy should be used in-context of a model.")
            }
        }
    }
}

/// Identifies auxiliary [output] types in OpenCRUD interface.
#[derive(Debug, Clone, PartialEq)]
pub enum AuxiliaryType {
    Edge,
    Connection,
}

impl NamedUnformatted for AuxiliaryType {
    fn name_str(&self, model_name_pc: &str) -> String {
        match self {
            AuxiliaryType::Edge => format!("{model_name_pc}Edge"),
            AuxiliaryType::Connection => format!("{model_name_pc}Connection"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    RootQuery,
    RootNode,
    PageInfo,
    Aggregate,
    Auxiliary(AuxiliaryType),
}

impl NamedUnformatted for QueryType {
    fn name_str(&self, model_name_pc: &str) -> String {
        match self {
            Self::Auxiliary(aux_type) => aux_type.name_str(model_name_pc),
            _ => panic!("{:#?} doesn't belong to any model.", self),
        }
    }
    fn common_name_str(&self) -> String {
        match self {
            Self::RootQuery => "Query".to_string(),
            Self::RootNode => "Node".to_string(),
            Self::PageInfo => "PageInfo".to_string(),
            Self::Aggregate => "Aggregate".to_string(),
            _ => panic!("{:#?} should be used in model context", self),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MutationType {
    RootMutation,
}

impl NamedUnformatted for MutationType {
    fn name_str(&self, _model_name_pc: &str) -> String {
        panic!("{:#?} doesn't belong to any model.", self)
    }
    fn common_name_str(&self) -> String {
        match self {
            Self::RootMutation => "Mutation",
        }
        .to_string()
    }
}

/// Identifies input types used in create interfaces.
#[derive(Debug, Clone, PartialEq)]
pub enum CreateInput {
    /// Identifies input type used to create a new object.
    /// Ex. UserCreateInput creates a new user.
    Create,
    /// Identifies the input type used to create the many objects in a relation
    /// in a nested create.
    /// Ex. PostCreateManyInlineInput will be used inside UserCreateInput
    /// to create posts inline when creating a new user.
    CreateManyInline,
    /// Identifies the input type used to create one object in one side of the relation
    /// in a nested create.
    /// Ex. ProfileCreateOneInlineInput will be used inside UserCreateInput
    /// to create user profile inline when creating a new user.
    CreateOneInline,
}

impl NamedUnformatted for CreateInput {
    fn name_str(&self, model_name_pc: &str) -> String {
        match self {
            CreateInput::Create => format!("{model_name_pc}CreateInput"),
            CreateInput::CreateManyInline => {
                format!("{model_name_pc}CreateManyInlineInput")
            }
            CreateInput::CreateOneInline => {
                format!("{model_name_pc}CreateOneInlineInput")
            }
        }
    }
}

/// Identifies input types used in update or upsert interfaces.
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateInput {
    /// Identifies the input type use to update a object.
    /// Ex. UserUpdateInput is used to capture
    /// the *complete data* to update a single user object including contained relations.
    Update,
    /// Identifies the input type used to update many objects in one go..
    /// Ex. UserUpdateManyInput is used to capture
    /// the non-relation, non-id data to update many objects.
    UpdateMany,
    /// Identifies the input type used to upsert a object.
    /// Ex. UserUpsertInput is used to capture the required data
    /// to update the *complete record* of a user including its contained relations,
    /// if the user exists, or to create the user if user doesn't exists.
    Upsert,
    /// Identifies the input type used to capture the data for many-side of the relation for updates.
    /// Ex. PostUpdateManyInlineInput[] is used inside UserUpdateInput to update many
    /// posts beloning to the user who is being updated.
    UpdateManyInline,
    /// Identifies the input type used to capture the data for one-side of the relation for updates.
    /// Ex. ProfileUpdateOneInlineInput is used inside UserUpdateInput to update the
    /// profile belonging to the user who is being updated.
    UpdateOneInline,
    /// Used inside UpdateManyInlineInput::update field to capture the updates to the many side of the relation
    /// where each update is accompanied with a unique where condition.
    /// Ex. UserUpdateWithNestedWhereUniqueInput is used inside UserUpdateManyInlineInput to update the
    /// user meeting the where unique condition when user is in a many side of the relationship.
    UpdateWithNestedWhereUnique,
    /// Used inside UpdateManyInlineInput::upsert field to capture the upserts to the many side of the relation
    /// where each update is accompanied with a unique where condition.
    /// Ex. UserUpsertWithNestedWhereUniqueInput is used inside UserUpdateManyInlineInput to upsert the
    /// user meeting the where unique condition when user is in a many side of the relationship.
    UpsertWithNestedWhereUnique,
    /// Identifies the input type specifying the existing object to connect to a relation.
    /// Ex. UserConnectInput is used inside UserUpdateManyInlineInput to connect existing users
    /// in a many side of relation.
    Connect,
    /// Identifies the input type which specifies the position from the list of connected objects,
    /// by-defult will add it to end of the list.
    ConnectPosition,
}

impl NamedUnformatted for UpdateInput {
    fn name_str(&self, model_name_pc: &str) -> String {
        match self {
            UpdateInput::Update => {
                format!("{model_name_pc}UpdateInput")
            }
            UpdateInput::Upsert => {
                format!("{model_name_pc}UpsertInput")
            }
            UpdateInput::UpdateMany => {
                format!("{model_name_pc}UpdateManyInput")
            }
            UpdateInput::UpdateManyInline => {
                format!("{model_name_pc}UpdateManyInlineInput")
            }
            UpdateInput::UpdateOneInline => {
                format!("{model_name_pc}UpdateOneInlineInput")
            }
            UpdateInput::UpdateWithNestedWhereUnique => {
                format!("{model_name_pc}UpdateWithNestedWhereUniqueInput")
            }
            UpdateInput::UpsertWithNestedWhereUnique => {
                format!("{model_name_pc}UpsertWithNestedWhereUniqueInput")
            }
            UpdateInput::Connect => {
                format!("{model_name_pc}ConnectInput")
            }
            UpdateInput::ConnectPosition => panic!("ConnectPositionInput is not specific to model, it is common for all models.")
        }
    }

    fn common_name_str(&self) -> String {
        match self {
            Self::ConnectPosition => "ConnectPositionInput".to_string(),
            _ =>  panic!("Common name for this abstraction is not available. This abstraction should be used in-conext of a specific model.")
        }
    }
}

/// Identifies the input types used in filters
#[derive(Debug, Clone, PartialEq)]
pub enum FilterInput {
    /// Identifies where critera where it can match one or more objects.
    Where,
    /// Idenifies the where critrial where it can match at most one object.
    WhereUnique,
}

impl NamedUnformatted for FilterInput {
    fn name_str(&self, model_name_pc: &str) -> String {
        match self {
            FilterInput::Where => format!("{model_name_pc}WhereInput"),
            FilterInput::WhereUnique => {
                format!("{model_name_pc}WhereUniqueInput")
            }
        }
    }
}
