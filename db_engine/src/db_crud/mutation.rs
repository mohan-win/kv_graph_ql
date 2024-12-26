use crate::errors::Error;
use async_trait::async_trait;

use super::*;

/// DB Mutation Interface.
#[async_trait]
pub trait DBMutation {
  /// Create and persist new object in DB.
  async fn create_object(
    &mut self,
    data: ObjectCreateInput,
  ) -> Result<Box<dyn DBObject>, Error>;
  /// Updates a single object, if found by the unique filter.
  /// And returns the updated object.
  async fn update_object(
    &mut self,
    r#where: ObjectWhereUniqueInput,
    data: ObjectUpdateInput,
  ) -> Result<Box<dyn DBObject>, Error>;
  /// Deletes a single object, if found by the unique filter.
  /// And returns the deleted object.
  async fn delete_object(
    &mut self,
    r#where: ObjectWhereUniqueInput,
  ) -> Result<Box<dyn DBObject>, Error>;
  /// Updates a single object if found by the unique filter or
  /// creates a new one. And returns either updated or newly created object.
  async fn upsert_object(
    &mut self,
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
  async fn delete_many_objects(
    &mut self,
    r#where: ObjectWhereInput,
    skip: u32,
    after: ID,
    first: u32,
    before: ID,
    last: u32,
  ) -> Result<ObjectConnection, Error>;
  /// Updates more than one objects found using the filter.
  /// and returns the updated objects.
  async fn update_many_objects(
    &mut self,
    r#where: ObjectWhereInput,
    skip: u32,
    after: ID,
    first: u32,
    before: ID,
    last: u32,
  ) -> Result<ObjectConnection, Error>;
}
