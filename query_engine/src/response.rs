use crate::{graphql_value::ConstValue, CacheControl, ServerError};
use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Response {
  /// Data for query result.
  #[serde(default)]
  pub data: ConstValue,

  /// Cache control value.
  #[serde(skip)]
  pub cache_control: CacheControl,

  /// Errors.
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub errors: Vec<ServerError>,

  /// HTTP headers.
  #[serde(skip)]
  pub http_headers: http::HeaderMap,
}

impl Response {
  /// Create a new successful response with data.
  #[must_use]
  pub fn new(data: impl Into<ConstValue>) -> Self {
    Self {
      data: data.into(),
      ..Default::default()
    }
  }

  /// Create a response from errros.
  #[must_use]
  pub fn from_errors(errors: Vec<ServerError>) -> Self {
    Self {
      errors,
      ..Default::default()
    }
  }

  /// Set the http headers of the response.
  #[must_use]
  pub fn http_headers(self, http_headers: http::HeaderMap) -> Self {
    Self {
      http_headers,
      ..self
    }
  }

  /// Set cache control of the response.
  #[must_use]
  pub fn cache_control(self, cache_control: CacheControl) -> Self {
    Self {
      cache_control,
      ..self
    }
  }

  /// Returns `true` if the response is ok.
  #[inline]
  pub fn is_ok(&self) -> bool {
    self.errors.is_empty()
  }

  /// Returns `true` if the response is error.
  #[inline]
  pub fn is_err(&self) -> bool {
    !self.is_ok()
  }

  /// Extract error from the response. Only if the `error` field is empty
  /// will this return `Ok`.
  #[inline]
  pub fn into_result(self) -> Result<Self, Vec<ServerError>> {
    if self.is_err() {
      Err(self.errors)
    } else {
      Ok(self)
    }
  }
}

/// Response for batchable queries.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum BatchResponse {
  /// Response for single query.
  Single(Response),
  /// Response for batch queries.
  Batch(Vec<Response>),
}

impl BatchResponse {
  /// Gets cache control value.
  pub fn cache_control(&self) -> CacheControl {
    match self {
      BatchResponse::Single(resp) => resp.cache_control,
      BatchResponse::Batch(resp) => {
        resp.iter().fold(CacheControl::default(), |acc, item| {
          acc.merge(&item.cache_control)
        })
      }
    }
  }

  /// Returns `true` if all responses are ok.
  pub fn is_ok(&self) -> bool {
    match self {
      BatchResponse::Single(resp) => resp.is_ok(),
      BatchResponse::Batch(resp) => resp.iter().all(Response::is_ok),
    }
  }

  /// Returns HTTP headers map.
  pub fn http_headers(&self) -> http::HeaderMap {
    match self {
      BatchResponse::Single(resp) => resp.http_headers.clone(),
      BatchResponse::Batch(resp) => {
        resp.iter().fold(http::HeaderMap::new(), |mut acc, item| {
          acc.extend(item.http_headers.clone());
          acc
        })
      }
    }
  }

  pub fn http_headers_iter(
    &self,
  ) -> impl Iterator<Item = (http::HeaderName, http::HeaderValue)> {
    let headers = self.http_headers();

    let mut current_name = None;
    headers.into_iter().filter_map(move |(name, value)| {
      if let Some(name) = name {
        current_name = Some(name);
      }

      current_name
        .clone()
        .map(|current_name| (current_name, value))
    })
  }
}

impl From<Response> for BatchResponse {
  fn from(resp: Response) -> Self {
    BatchResponse::Single(resp)
  }
}

impl From<Vec<Response>> for BatchResponse {
  fn from(responses: Vec<Response>) -> Self {
    BatchResponse::Batch(responses)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_batch_response_single() {
    let resp = BatchResponse::Single(Response::new(ConstValue::Boolean(true)));
    assert_eq!(serde_json::to_string(&resp).unwrap(), r#"{"data":true}"#);
  }

  #[test]
  fn test_batch_response_batch() {
    let resp = BatchResponse::Batch(vec![
      Response::new(ConstValue::Boolean(true)),
      Response::new(ConstValue::String("1".to_string())),
    ]);
    assert_eq!(
      serde_json::to_string(&resp).unwrap(),
      r#"[{"data":true},{"data":"1"}]"#
    );
  }
}
