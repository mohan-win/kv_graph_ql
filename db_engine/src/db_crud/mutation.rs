use crate::errors::Error;

use super::*;

/// DB Mutation Interface.
pub trait DBMutation {
  /// Create and persist new object in DB.
  fn create_object(data: ObjectCreateInput) -> Result<Box<dyn DBObject>, Error>;
  /// Updates a single object, if found by the unique filter.
  /// And returns the updated object.
  fn update_object(
    r#where: ObjectWhereUniqueInput,
    data: ObjectUpdateInput,
  ) -> Result<Box<dyn DBObject>, Error>;
  /// Deletes a single object, if found by the unique filter.
  /// And returns the deleted object.
  fn delete_object(r#where: ObjectWhereUniqueInput) -> Result<Box<dyn DBObject>, Error>;
  /// Updates a single object if found by the unique filter or
  /// creates a new one. And returns either updated or newly created object.
  fn upsert_object(
    r#where: ObjectWhereUniqueInput,
    data: ObjectUpsertInput,
  ) -> Result<Box<dyn DBObject>, Error>;
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
