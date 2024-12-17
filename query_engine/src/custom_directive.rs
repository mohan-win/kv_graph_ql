use std::{borrow::Cow, future::Future, task::Context};

use crate::{
  graphql_parser::types::Directive, graphql_value::ConstValue, registry::Registry,
  ContextDirective, ServerResult,
};

pub type ResolveFut<'a> =
  &'a mut (dyn Future<Output = ServerResult<Option<ConstValue>>> + Send + Unpin);

#[doc(hidden)]
pub trait CustomDirectiveFactory: Send + Sync + 'static {
  fn name(&self) -> Cow<'static, str>;

  fn register(&self, registry: &mut Registry);

  fn create(
    &self,
    ctx: &ContextDirective<'_>,
    directive: &Directive,
  ) -> ServerResult<Box<dyn CustomDirective>>;
}

/// Minimal data required to register directive into registry.
#[doc(hidden)]
pub trait TypeDirective {
  fn name(&self) -> Cow<'static, str>;

  fn register(&self, registry: &mut Registry);
}

/// Represents a custom directive.
#[async_trait::async_trait]
pub trait CustomDirective: Send + Sync + 'static {
  async fn resolve_field(
    &self,
    ctx: &Context<'_>,
    resolve: ResolveFut<'_>,
  ) -> ServerResult<Option<ConstValue>> {
    resolve.await
  }
}
