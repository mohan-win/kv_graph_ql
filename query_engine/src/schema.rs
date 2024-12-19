use std::{collections::HashMap, ops::Deref, sync::Arc};

use crate::{
  context::Data, registry::Registry, validation::ValidationMode, CustomDirectiveFactory,
  DataModel,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum IntrospectionMode {
  /// Introspection only.
  IntrospectionOnly,
  /// Enables introspection.
  #[default]
  Enabled,
  /// Disables introspection.
  Disabled,
}

#[doc(hidden)]
pub struct SchemaEnvInner {
  pub data_model: DataModel,
  pub registry: Registry,
  pub data: Data,
  pub custom_directives: HashMap<String, Box<dyn CustomDirectiveFactory>>,
}

#[doc(hidden)]
#[derive(Clone)]
pub struct SchemaEnv(pub(crate) Arc<SchemaEnvInner>);

impl Deref for SchemaEnv {
  type Target = SchemaEnvInner;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
