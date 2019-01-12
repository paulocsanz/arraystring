//! Draining iterator for [`ArrayString`]
//!
//! [`ArrayString`]: ../struct.ArrayString.html

use crate::prelude::*;
use core::iter::FusedIterator;

/// A draining iterator for [`ArrayString`].
///
/// Created through [`drain`]
///
/// [`ArrayString`]: ../struct.ArrayString.html
/// [`drain`]: ../struct.ArrayString.html#method.drain
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Drain<S: Length>(pub(crate) ArrayString<S>, pub(crate) u8);

impl<SIZE: Length + Copy> Copy for Drain<SIZE> where SIZE::Array: Copy {}

impl<S: Length> Drain<S> {
    /// Extracts string slice containing the remaining characters of `Drain`.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<S: Length> Iterator for Drain<S> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .as_str()
            .get(self.1.into()..)
            .and_then(|s| s.chars().next())
            .map(|c| {
                self.1 = self.1.saturating_add(c.len_utf8() as u8);
                c
            })
    }
}

impl<S: Length> DoubleEndedIterator for Drain<S> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<S: Length> FusedIterator for Drain<S> {}
