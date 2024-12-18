#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CacheControl {
  /// Scope is public, default is true.
  pub public: bool,

  /// Cache max age, `-1` represents `no-cache`, default is `0`.
  pub max_age: i32,
}

impl Default for CacheControl {
  fn default() -> Self {
    Self {
      public: true,
      max_age: 0,
    }
  }
}

impl CacheControl {
  /// Get 'Cache-Control' header value.
  #[must_use]
  pub fn value(&self) -> Option<String> {
    let mut value = if self.max_age > 0 {
      format!("max-age={}", self.max_age)
    } else if self.max_age == -1 {
      "no-cache".to_string()
    } else {
      String::new()
    };

    if !self.public {
      if !value.is_empty() {
        value += ", ";
      }
      value += "private";
    }

    if !value.is_empty() {
      Some(value)
    } else {
      None
    }
  }
}

impl CacheControl {
  #[must_use]
  pub(crate) fn merge(self, other: &CacheControl) -> CacheControl {
    CacheControl {
      public: self.public && other.public,
      max_age: match (self.max_age, other.max_age) {
        (-1, _) => -1,
        (_, -1) => -1,
        (a, 0) => a,
        (0, b) => b,
        (a, b) => a.min(b),
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn to_value() {
    assert_eq!(
      CacheControl {
        public: true,
        max_age: 0,
      }
      .value(),
      None
    );

    assert_eq!(
      CacheControl {
        public: false,
        max_age: 0,
      }
      .value(),
      Some("private".to_string())
    );

    assert_eq!(
      CacheControl {
        public: false,
        max_age: 10,
      }
      .value(),
      Some("max-age=10, private".to_string())
    );

    assert_eq!(
      CacheControl {
        public: true,
        max_age: 10,
      }
      .value(),
      Some("max-age=10".to_string())
    );

    assert_eq!(
      CacheControl {
        public: true,
        max_age: -1,
      }
      .value(),
      Some("no-cache".to_string())
    );

    assert_eq!(
      CacheControl {
        public: false,
        max_age: -1,
      }
      .value(),
      Some("no-cache, private".to_string())
    );
  }
}
