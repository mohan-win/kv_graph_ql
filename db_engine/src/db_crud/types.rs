use std::sync::Arc;

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
