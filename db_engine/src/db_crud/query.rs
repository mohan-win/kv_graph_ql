use crate::errors::Error;
use async_trait::async_trait;

use super::*;

/// DB Query interface.
#[async_trait]
pub trait DBQuery {
  async fn get_object(
    &self,
    r#where: ObjectWhereUniqueInput,
  ) -> Result<Box<dyn DBObject>, Error>;
  async fn get_objects(
    &self,
    r#where: ObjectWhereInput,
    order_by: ObjectOrderByInput,
    skip: u32,
    after: ID,
    first: u32,
    before: ID,
    last: u32,
  ) -> Result<ObjectConnection, Error>;
}
