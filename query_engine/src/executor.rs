use std::future::Future;

use crate::{BatchRequest, BatchResponse, Data, Request, Response};

use futures_util::{stream::FuturesOrdered, StreamExt};

pub trait Executor: Unpin + Clone + Send + Sync + 'static {
  fn execute(&self, request: Request) -> impl Future<Output = Response> + Send;

  fn execute_batch(
    &self,
    batch_request: BatchRequest,
  ) -> impl Future<Output = BatchResponse> + Send {
    async {
      match batch_request {
        BatchRequest::Single(request) => {
          BatchResponse::Single(self.execute(request).await)
        }
        BatchRequest::Batch(requests) => BatchResponse::Batch(
          FuturesOrdered::from_iter(
            requests.into_iter().map(|request| self.execute(request)),
          )
          .collect()
          .await,
        ),
      }
    }
  }
}
