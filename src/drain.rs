//! Draining iterator for [`ArrayString`]
//!
//! [`ArrayString`]: ../struct.ArrayString.html

use crate::{arraystring::sealed::ValidCapacity, prelude::*};
use core::fmt::{self, Debug, Formatter};
use core::{cmp::Ordering, hash::Hash, hash::Hasher, iter::FusedIterator};

/// A draining iterator for [`ArrayString`].
///
/// Created through [`drain`]
///
/// [`ArrayString`]: ../struct.ArrayString.html
/// [`drain`]: ../struct.ArrayString.html#method.drain
pub struct Drain<const N: usize>(pub(crate) ArrayString<N>)
where
    ArrayString<N>: ValidCapacity;

impl<const N: usize> Copy for Drain<N> where ArrayString<N>: ValidCapacity {}
impl<const N: usize> Clone for Drain<N>
where
    ArrayString<N>: ValidCapacity,
{
    fn clone(&self) -> Self {
        Self(self.0)
    }
}
impl<const N: usize> Default for Drain<N>
where
    ArrayString<N>: ValidCapacity,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<const N: usize> Debug for Drain<N>
where
    ArrayString<N>: ValidCapacity,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_tuple("Drain")
            .field(&self.0)
            .finish()
    }
}

impl<const N: usize> PartialEq for Drain<N>
where
    ArrayString<N>: ValidCapacity,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(other.as_str())
    }
}
impl<const N: usize> Eq for Drain<N> where ArrayString<N>: ValidCapacity {}

impl<const N: usize> Ord for Drain<N>
where
    ArrayString<N>: ValidCapacity,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<const N: usize> PartialOrd for Drain<N>
where
    ArrayString<N>: ValidCapacity,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize> Hash for Drain<N>
where
    ArrayString<N>: ValidCapacity,
{
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.as_str().hash(hasher)
    }
}

impl<const N: usize> Drain<N>
where
    ArrayString<N>: ValidCapacity,
{
    /// Extracts string slice containing the remaining characters of `Drain`.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<const N: usize> Iterator for Drain<N>
where
    ArrayString<N>: ValidCapacity,
{
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.remove(0).ok()
    }
}

impl<const N: usize> DoubleEndedIterator for Drain<N>
where
    ArrayString<N>: ValidCapacity,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<const N: usize> FusedIterator for Drain<N> where ArrayString<N>: ValidCapacity {}
