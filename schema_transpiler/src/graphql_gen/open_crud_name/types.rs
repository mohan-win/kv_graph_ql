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
    Id,
    Query(QueryType),
    Create(CreateInputType),
    Update(UpdateInputType),
    Filter(FilterInputType),
    OrderByInput,
}

impl NamedUnformatted for OpenCRUDType {
    fn name_str(&self, model_name_pc: &str) -> String {
        match self {
            OpenCRUDType::Id => panic!("ID type is not model specific."),
            OpenCRUDType::Query(query_type) => query_type.name_str(model_name_pc),
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
            OpenCRUDType::Id => "ID".to_string(),
            OpenCRUDType::Query(query_type) => query_type.common_name_str(),
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

impl NamedUnformatted for CreateInputType {
    fn name_str(&self, model_name_pc: &str) -> String {
        match self {
            CreateInputType::CreateInput => format!("{model_name_pc}CreateInput"),
            CreateInputType::CreateManyInlineInput => {
                format!("{model_name_pc}CreateManyInlineInput")
            }
            CreateInputType::CreateOneInlineInput => {
                format!("{model_name_pc}CreateOneInlineInput")
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
    /// profile belonging to the user who is being updated.
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

impl NamedUnformatted for UpdateInputType {
    fn name_str(&self, model_name_pc: &str) -> String {
        match self {
            UpdateInputType::UpdateInput => {
                format!("{model_name_pc}UpdateInput")
            }
            UpdateInputType::UpsertInput => {
                format!("{model_name_pc}UpsertInput")
            }
            UpdateInputType::UpdateManyInlineInput => {
                format!("{model_name_pc}UpdateManyInlineInput")
            }
            UpdateInputType::UpdateOneInlineInput => {
                format!("{model_name_pc}UpdateOneInlineInput")
            }
            UpdateInputType::UpdateWithNestedWhereUniqueInput => {
                format!("{model_name_pc}UpdateWithNestedWhereUniqueInput")
            }
            UpdateInputType::UpsertWithNestedWhereUniqueInput => {
                format!("{model_name_pc}UpsertWithNestedWhereUniqueInput")
            }
            UpdateInputType::ConnectInput => {
                format!("{model_name_pc}ConnectInput")
            }
        }
    }
}

/// Identifies the input types used in filters
#[derive(Debug, Clone, PartialEq)]
pub enum FilterInputType {
    /// Identifies where critera where it can match one or more objects.
    WhereInput,
    /// Idenifies the where critrial where it can match at most one object.
    WhereUniqueInput,
}

impl NamedUnformatted for FilterInputType {
    fn name_str(&self, model_name_pc: &str) -> String {
        match self {
            FilterInputType::WhereInput => format!("{model_name_pc}WhereInput"),
            FilterInputType::WhereUniqueInput => {
                format!("{model_name_pc}WhereUniqueInput")
            }
        }
    }
}
