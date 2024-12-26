use graphql_value::{ConstValue as Value, Name};
use std::sync::Arc;

/// Errors.
pub enum Error {}

/// ID of an object.
pub struct ID(Arc<str>);

/// Unique filter to search and find at most a object.
pub struct ObjectWhereUniqueInput {}

/// Filter to search and find more than one objects.
pub struct ObjectWhereInput {}

/// Objects order in result.
pub enum ObjectOrderByInput {}

/// All inputs needed to create a single object.
pub struct ObjectCreateInput {}

/// Type to capture the update data to update a single object.
pub struct ObjectUpdateInput {}

/// Type to capture the upsert data to either create or update a single object.
pub struct ObjectUpsertInput {}

/// Container to capture array objects, along with pagniation data.
pub struct ObjectConnection {}

/// An Object persisted in DB should expose these traits.
pub trait Object {
  /// Get object's ID.
  fn id(&self) -> ID;
  /// Retrieve the value of the object's field.
  fn field(&self, name: Name) -> Value;
  /// Retrieve a single relation stored in the object's field of given name.
  fn relation(&self, name: Name) -> Result<Box<dyn Object>, Error>;
  /// Retrieve array of relations stored in the object's field of the given name.
  fn relations(
    &self,
    name: Name,
    r#where: ObjectWhereInput,
    order_by: ObjectOrderByInput,
    skip: u32,
    after: ID,
    first: u32,
    before: ID,
    last: u32,
  ) -> Result<ObjectConnection, Error>;
}

/// DB Query interface.
pub trait DBQuery {
  fn get_object(r#where: ObjectWhereUniqueInput) -> Result<Box<dyn Object>, Error>;
  fn get_objects(
    r#where: ObjectWhereInput,
    order_by: ObjectOrderByInput,
    skip: u32,
    after: ID,
    first: u32,
    before: ID,
    last: u32,
  ) -> Result<ObjectConnection, Error>;
}

/// DB Mutation Interface.
pub trait DBMutation {
  /// Create and persist new object in DB.
  fn create_object(data: ObjectCreateInput) -> Result<Box<dyn Object>, Error>;
  /// Updates a single object, if found by the unique filter.
  /// And returns the updated object.
  fn update_object(
    r#where: ObjectWhereUniqueInput,
    data: ObjectUpdateInput,
  ) -> Result<Box<dyn Object>, Error>;
  /// Deletes a single object, if found by the unique filter.
  /// And returns the deleted object.
  fn delete_object(r#where: ObjectWhereUniqueInput) -> Result<Box<dyn Object>, Error>;
  /// Updates a single object if found by the unique filter or
  /// creates a new one. And returns either updated or newly created object.
  fn upsert_object(
    r#where: ObjectWhereUniqueInput,
    data: ObjectUpsertInput,
  ) -> Result<Box<dyn Object>, Error>;
  /// Deletes more than one objects found using the filter.
  /// And returns the deleted objects.
  /// Note: It means we should only `mark objects as delete` and never remove from DB
  /// immmediately.
  /// # ToDo::
  /// Figure if we need to purge these deleted objects from DB ever ?
  /// How it can impact migration.
  fn delete_many_objects(
    r#where: ObjectWhereInput,
    skip: u32,
    after: ID,
    first: u32,
    before: ID,
    last: u32,
  ) -> Result<ObjectConnection, Error>;
  /// Updates more than one objects found using the filter.
  /// and returns the updated objects.
  fn update_many_objects(
    r#where: ObjectWhereInput,
    skip: u32,
    after: ID,
    first: u32,
    before: ID,
    last: u32,
  ) -> Result<ObjectConnection, Error>;
}
