//! Draining iterator for [`ArrayString`]
///
/// [`ArrayString`]: ./trait.ArrayString.html

use crate::prelude::*;
use core::iter::FusedIterator;
 
/// A draining iterator for [`ArrayString`].
///
/// Created through [`drain`]
///
/// [`ArrayString`]: ./trait.ArrayString.html
/// [`ArrayString`]: ./struct.ArrayString.html#method.drain
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Drain<S: ArrayLength<u8>>(pub(crate) ArrayString<S>, pub(crate) u8);

impl<SIZE: ArrayLength<u8> + Copy> Copy for Drain<SIZE> where SIZE::ArrayType: Copy {}

impl<S: ArrayLength<u8>> Drain<S> {
    /// Extracts string slice containing the entire `Drain`.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<S: ArrayLength<u8>> Iterator for Drain<S> {
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

impl<S: ArrayLength<u8>> DoubleEndedIterator for Drain<S> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<S: ArrayLength<u8>> FusedIterator for Drain<S> {}
