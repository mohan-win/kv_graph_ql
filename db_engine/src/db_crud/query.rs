use crate::errors::Error;

use super::*;

/// DB Query interface.
pub trait DBQuery {
  fn get_object(r#where: ObjectWhereUniqueInput) -> Result<Box<dyn DBObject>, Error>;
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
