use crate::errors::Error;

use super::*;
use graphql_value::{ConstValue as Value, Name};

/// An Object persisted in DB should expose these traits.
pub trait DBObject {
  /// Get object's ID.
  fn id(&self) -> ID;
  /// Retrieve the value of the object's field.
  fn field(&self, name: Name) -> Value;
  /// Retrieve a single relation stored in the object's field of given name.
  fn relation(&self, name: Name) -> Result<Box<dyn DBObject>, Error>;
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
