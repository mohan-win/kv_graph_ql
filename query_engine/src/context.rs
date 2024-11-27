use std::fmt::Display;

use serde::{ser::SerializeSeq, Serialize};

/// A segment in the path to the current query.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(untagged)]
pub enum QueryPathSegment<'a> {
  /// We are currently resolving an element in a list.
  Index(usize),
  /// We are currently resolving a field in an object.
  Name(&'a str),
}

/// A path to the current query.
///
/// The path is stored as a kind of reverse linked list
pub struct QueryPathNode<'a> {
  /// The parent node to this, if there is one.
  pub parent: Option<&'a QueryPathNode<'a>>,

  /// The current path segment being resolved.
  pub segment: QueryPathSegment<'a>,
}

impl<'a> serde::Serialize for QueryPathNode<'a> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut seq = serializer.serialize_seq(None)?;
    self.try_for_each(|segment| seq.serialize_element(segment))?;
    seq.end()
  }
}

impl<'a> Display for QueryPathNode<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut first = true;
    self.try_for_each(|segment| {
      if !first {
        write!(f, ".")?;
      }
      first = false;

      match segment {
        QueryPathSegment::Index(idx) => write!(f, "{}", *idx),
        QueryPathSegment::Name(name) => write!(f, "{}", name),
      }
    })
  }
}

impl<'a> QueryPathNode<'a> {
  /// Get the current field name.
  ///
  /// This traverses all the parents of the node until it finds one that is a
  /// field name.
  pub fn field_name(&self) -> &str {
    std::iter::once(self)
      .chain(self.parents())
      .find_map(|node| match node.segment {
        QueryPathSegment::Name(name) => Some(name),
        QueryPathSegment::Index(_) => None,
      })
      .unwrap()
  }

  /// Get the path represented by `Vec<String>`; numbers will be stringified.
  #[must_use]
  pub fn to_string_vec(self) -> Vec<String> {
    let mut res = Vec::new();
    self.for_each(|s| {
      res.push(match s {
        QueryPathSegment::Name(name) => (*name).to_string(),
        QueryPathSegment::Index(idx) => idx.to_string(),
      });
    });
    res
  }

  pub fn parents(&self) -> Parents<'_> {
    Parents(self)
  }

  pub(crate) fn for_each<F: FnMut(&QueryPathSegment<'a>)>(&self, mut f: F) {
    let _ = self.try_for_each::<std::convert::Infallible, _>(|segment| {
      f(segment);
      Ok(())
    });
  }

  pub(crate) fn try_for_each<E, F: FnMut(&QueryPathSegment<'a>) -> Result<(), E>>(
    &self,
    mut f: F,
  ) -> Result<(), E> {
    self.try_for_each_ref(&mut f)
  }

  fn try_for_each_ref<E, F: FnMut(&QueryPathSegment<'a>) -> Result<(), E>>(
    &self,
    f: &mut F,
  ) -> Result<(), E> {
    if let Some(parent) = &self.parent {
      parent.try_for_each_ref(f)?;
    }
    f(&self.segment)
  }
}

/// An iterator over the parents of a
/// QueryPathNode
pub struct Parents<'a>(&'a QueryPathNode<'a>);

impl<'a> Parents<'a> {
  /// Get the current query path node, which the call to `next` will
  /// get the parents of.
  pub fn current(&self) -> &'a QueryPathNode<'a> {
    self.0
  }
}

impl<'a> Iterator for Parents<'a> {
  type Item = &'a QueryPathNode<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    let parent = self.0.parent;
    if let Some(parent) = parent {
      self.0 = parent;
    }
    parent
  }
}

impl<'a> std::iter::FusedIterator for Parents<'a> {}
