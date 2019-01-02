use core::ops::{
    Add, Deref, DerefMut, Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo,
    RangeToInclusive,
};
use crate::prelude::*;

impl<SIZE: ArrayLength<u8>> core::str::FromStr for ArrayString<SIZE> {
    type Err = OutOfBounds;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(s)
    }
}

impl<SIZE: ArrayLength<u8>> core::fmt::Debug for ArrayString<SIZE> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("ArrayString")
            .field("array", &self.as_str())
            .field("size", &self.size)
            .finish()
    }
}

impl<'a, 'b, SIZE: ArrayLength<u8>> PartialEq<str> for ArrayString<SIZE> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl<SIZE: ArrayLength<u8>> core::borrow::Borrow<str> for ArrayString<SIZE> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<SIZE: ArrayLength<u8>> core::hash::Hash for ArrayString<SIZE> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, hasher: &mut H) {
        self.as_str().hash(hasher);
    }
}

impl<SIZE: ArrayLength<u8>> PartialEq for ArrayString<SIZE> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(other.as_str())
    }
}
impl<SIZE: ArrayLength<u8>> Eq for ArrayString<SIZE> {}

impl<SIZE: ArrayLength<u8>> Ord for ArrayString<SIZE> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<SIZE: ArrayLength<u8>> PartialOrd for ArrayString<SIZE> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, SIZE: ArrayLength<u8>> Add<&'a str> for ArrayString<SIZE> {
    type Output = Self;

    #[inline]
    fn add(self, other: &str) -> Self::Output {
        let mut out = Self::default();
        unsafe { out.push_str_unchecked(self.as_str()) };
        out.push_str(other);
        out
    }
}

impl<SIZE: ArrayLength<u8>> core::fmt::Write for ArrayString<SIZE> {
    #[inline]
    fn write_str(&mut self, slice: &str) -> core::fmt::Result {
        self.try_push_str(slice).map_err(|_| core::fmt::Error)
    }
}

impl<SIZE: ArrayLength<u8>> core::fmt::Display for ArrayString<SIZE> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<SIZE: ArrayLength<u8>> Deref for ArrayString<SIZE> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<SIZE: ArrayLength<u8>> DerefMut for ArrayString<SIZE> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<SIZE: ArrayLength<u8>> IndexMut<RangeFrom<u8>> for ArrayString<SIZE> {
    #[inline]
    fn index_mut(&mut self, index: RangeFrom<u8>) -> &mut str {
        let start = index.start as usize;
        let start = RangeFrom { start };
        self.as_mut_str().index_mut(start)
    }
}

impl<SIZE: ArrayLength<u8>> IndexMut<RangeTo<u8>> for ArrayString<SIZE> {
    #[inline]
    fn index_mut(&mut self, index: RangeTo<u8>) -> &mut str {
        let end = index.end as usize;
        self.as_mut_str().index_mut(RangeTo { end })
    }
}

impl<SIZE: ArrayLength<u8>> IndexMut<RangeFull> for ArrayString<SIZE> {
    #[inline]
    fn index_mut(&mut self, index: RangeFull) -> &mut str {
        self.as_mut_str().index_mut(index)
    }
}

impl<SIZE: ArrayLength<u8>> IndexMut<Range<u8>> for ArrayString<SIZE> {
    #[inline]
    fn index_mut(&mut self, index: Range<u8>) -> &mut str {
        let (start, end) = (index.start as usize, index.end as usize);
        let range = Range { start, end };
        self.as_mut_str().index_mut(range)
    }
}

impl<SIZE: ArrayLength<u8>> IndexMut<RangeToInclusive<u8>> for ArrayString<SIZE> {
    #[inline]
    fn index_mut(&mut self, index: RangeToInclusive<u8>) -> &mut str {
        let end = index.end as usize;
        let range = RangeToInclusive { end };
        self.as_mut_str().index_mut(range)
    }
}

impl<SIZE: ArrayLength<u8>> IndexMut<RangeInclusive<u8>> for ArrayString<SIZE> {
    #[inline]
    fn index_mut(&mut self, index: RangeInclusive<u8>) -> &mut str {
        let (start, end) = (*index.start() as usize, *index.end() as usize);
        let range = RangeInclusive::new(start, end);
        self.as_mut_str().index_mut(range)
    }
}

impl<SIZE: ArrayLength<u8>> Index<RangeFrom<u8>> for ArrayString<SIZE> {
    type Output = str;

    #[inline]
    fn index(&self, index: RangeFrom<u8>) -> &Self::Output {
        let start = index.start as usize;
        self.as_str().index(RangeFrom { start })
    }
}

impl<SIZE: ArrayLength<u8>> Index<RangeTo<u8>> for ArrayString<SIZE> {
    type Output = str;

    #[inline]
    fn index(&self, index: RangeTo<u8>) -> &Self::Output {
        let end = index.end as usize;
        self.as_str().index(RangeTo { end })
    }
}

impl<SIZE: ArrayLength<u8>> Index<RangeFull> for ArrayString<SIZE> {
    type Output = str;

    #[inline]
    fn index(&self, index: RangeFull) -> &Self::Output {
        self.as_str().index(index)
    }
}

impl<SIZE: ArrayLength<u8>> Index<Range<u8>> for ArrayString<SIZE> {
    type Output = str;

    #[inline]
    fn index(&self, index: Range<u8>) -> &Self::Output {
        let (start, end) = (index.start as usize, index.end as usize);
        self.as_str().index(Range { start, end })
    }
}

impl<SIZE: ArrayLength<u8>> Index<RangeToInclusive<u8>> for ArrayString<SIZE> {
    type Output = str;

    #[inline]
    fn index(&self, index: RangeToInclusive<u8>) -> &Self::Output {
        let end = index.end as usize;
        self.as_str().index(RangeToInclusive { end })
    }
}

impl<SIZE: ArrayLength<u8>> Index<RangeInclusive<u8>> for ArrayString<SIZE> {
    type Output = str;

    #[inline]
    fn index(&self, index: RangeInclusive<u8>) -> &Self::Output {
        let (start, end) = (*index.start() as usize, *index.end() as usize);
        let range = RangeInclusive::new(start, end);
        self.as_str().index(range)
    }
}

