/// Abstracts [`ArrayString`] implementation
///
/// [`ArrayString`]: ./array/trait.ArrayString.html
///
/// ```rust
/// # #[macro_use]
/// # extern crate arraystring;
/// # use std::mem::size_of;
/// # fn main() {
/// impl_string!(pub struct Username(20));
/// impl_string!(struct PasswordHash(60));
/// impl_string!(struct TinyString(5));
///
/// // Uses one byte more than string size (length)
/// assert_eq!(size_of::<Username>(), 21);
/// # }
/// ```
#[macro_export]
macro_rules! impl_string {
    ($(#[$attr:meta])* pub struct $name: ident ($size: expr)) => {
        /// Customized stack based string
        #[derive(Copy, Clone)]
        #[allow(trivial_numeric_casts)]
        $(#[$attr:meta])*
        pub struct $name([u8; $size as usize], $crate::Size);
        __inner_impl_string!($name, $size);
    };
    ($(#[$attr:meta])* struct $name: ident ($size: expr)) => {
        /// Customized stack based string
        #[derive(Copy, Clone)]
        #[allow(trivial_numeric_casts)]
        $(#[$attr:meta])*
        struct $name([u8; $size as usize], $crate::Size);
        __inner_impl_string!($name, $size);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __inner_impl_string {
    ($name: ident, $size: expr) => {
        #[allow(unused_imports)]
        use $crate::prelude::*;
        impl $crate::array::ArrayBuffer for $name {
            #[inline]
            unsafe fn buffer(&mut self) -> &mut [u8] {
                self.0.as_mut()
            }

            #[inline]
            fn add_assign_len(&mut self, v: Size) {
                self.1 = self.1.saturating_add(v);
            }

            #[inline]
            fn sub_assign_len(&mut self, v: Size) {
                self.1 = self.1.saturating_sub(v);
            }

            #[inline]
            fn replace_len(&mut self, v: Size) {
                self.1 = v;
            }

            #[inline]
            fn get_len(&self) -> Size {
                self.1
            }
        }

        impl Default for $name {
            #[inline]
            fn default() -> Self {
                #[allow(trivial_numeric_casts)]
                $name([0; $size as usize], 0)
            }
        }

        impl AsRef<str> for $name {
            #[inline]
            fn as_ref(&self) -> &str {
                unsafe { $crate::core::str::from_utf8_unchecked(self.as_ref()) }
            }
        }

        impl AsMut<str> for $name {
            #[inline]
            fn as_mut(&mut self) -> &mut str {
                use $crate::array::ArrayBuffer;
                let len = self.get_len() as usize;
                let slice = unsafe { self.0.get_unchecked_mut(..len) };
                unsafe { $crate::core::str::from_utf8_unchecked_mut(slice) }
            }
        }

        impl AsRef<[u8]> for $name {
            #[inline]
            fn as_ref(&self) -> &[u8] {
                use $crate::array::ArrayBuffer;
                unsafe { self.0.get_unchecked(..self.get_len() as usize) }
            }
        }

        impl $crate::ArrayString for $name {
            const SIZE: $crate::Size = $size;
        }

        /// Creates new `ArrayString` from string slice if length is lower or equal than `SIZE`, otherwise returns a error.
        ///
        /// ```rust
        /// # use arraystring::{error::Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// let string = CacheString::try_from_str("My String")?;
        /// assert_eq!(string.as_str(), "My String");
        ///
        /// assert_eq!(CacheString::try_from_str("")?.as_str(), "");
        ///
        /// let out_of_bounds = "0".repeat(CacheString::SIZE as usize + 1);
        /// assert!(CacheString::try_from_str(&out_of_bounds).is_err());
        /// # Ok(())
        /// # }
        /// ```
        impl $crate::core::str::FromStr for $name {
            type Err = $crate::error::OutOfBoundsError;

            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::try_from_str(s)
            }
        }

        impl $crate::core::fmt::Debug for $name {
            #[inline]
            fn fmt(&self, f: &mut $crate::core::fmt::Formatter) -> $crate::core::fmt::Result {
                use $crate::array::ArrayBuffer;
                let s: &str = self.as_ref();
                f.debug_tuple(stringify!($name))
                    .field(&s)
                    .field(&self.get_len())
                    .finish()
            }
        }

        impl<'a, 'b> PartialEq<str> for $name {
            #[inline]
            fn eq(&self, other: &str) -> bool {
                self.as_str().eq(other)
            }
        }

        impl $crate::core::borrow::Borrow<str> for $name {
            #[inline]
            fn borrow(&self) -> &str {
                self.as_str()
            }
        }

        impl $crate::core::hash::Hash for $name {
            #[inline]
            fn hash<H: $crate::core::hash::Hasher>(&self, hasher: &mut H) {
                self.as_str().hash(hasher);
            }
        }

        impl PartialEq for $name {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.as_str().eq(other.as_str())
            }
        }
        impl Eq for $name {}

        impl Ord for $name {
            #[inline]
            fn cmp(&self, other: &Self) -> $crate::core::cmp::Ordering {
                self.as_str().cmp(other.as_str())
            }
        }

        impl PartialOrd for $name {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<$crate::core::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl<'a> $crate::core::ops::Add<&'a str> for $name {
            type Output = Self;

            #[inline]
            fn add(self, other: &str) -> Self::Output {
                let mut out = Self::default();
                unsafe { out.push_str_unchecked(self.as_str()) };
                out.push_str(other);
                out
            }
        }

        impl $crate::core::fmt::Write for $name {
            #[inline]
            fn write_str(&mut self, slice: &str) -> $crate::core::fmt::Result {
                self.try_push_str(slice).map_err(|_| $crate::core::fmt::Error)
            }
        }

        impl $crate::core::fmt::Display for $name {
            #[inline]
            fn fmt(&self, f: &mut $crate::core::fmt::Formatter) -> $crate::core::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl $crate::core::ops::Deref for $name {
            type Target = str;

            #[inline]
            fn deref(&self) -> &Self::Target {
                self.as_ref()
            }
        }

        impl $crate::core::ops::DerefMut for $name {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.as_mut()
            }
        }

        impl $crate::core::ops::IndexMut<$crate::core::ops::RangeFrom<Size>> for $name {
            #[inline]
            fn index_mut(&mut self, index: $crate::core::ops::RangeFrom<Size>) -> &mut str {
                let start = index.start as usize;
                let start = $crate::core::ops::RangeFrom { start };
                self.as_str_mut().index_mut(start)
            }
        }

        impl $crate::core::ops::IndexMut<$crate::core::ops::RangeTo<Size>> for $name {
            #[inline]
            fn index_mut(&mut self, index: $crate::core::ops::RangeTo<Size>) -> &mut str {
                let end = index.end as usize;
                self.as_str_mut()
                    .index_mut($crate::core::ops::RangeTo { end })
            }
        }

        impl $crate::core::ops::IndexMut<$crate::core::ops::RangeFull> for $name {
            #[inline]
            fn index_mut(&mut self, index: $crate::core::ops::RangeFull) -> &mut str {
                self.as_str_mut().index_mut(index)
            }
        }

        impl $crate::core::ops::IndexMut<$crate::core::ops::Range<Size>> for $name {
            #[inline]
            fn index_mut(&mut self, index: $crate::core::ops::Range<Size>) -> &mut str {
                let (start, end) = (index.start as usize, index.end as usize);
                let range = $crate::core::ops::Range { start, end };
                self.as_str_mut().index_mut(range)
            }
        }

        impl $crate::core::ops::IndexMut<$crate::core::ops::RangeToInclusive<Size>> for $name {
            #[inline]
            fn index_mut(&mut self, index: $crate::core::ops::RangeToInclusive<Size>) -> &mut str {
                let end = index.end as usize;
                let range = $crate::core::ops::RangeToInclusive { end };
                self.as_str_mut().index_mut(range)
            }
        }

        impl $crate::core::ops::IndexMut<$crate::core::ops::RangeInclusive<Size>> for $name {
            #[inline]
            fn index_mut(&mut self, index: $crate::core::ops::RangeInclusive<Size>) -> &mut str {
                let (start, end) = (*index.start() as usize, *index.end() as usize);
                let range = $crate::core::ops::RangeInclusive::new(start, end);
                self.as_str_mut().index_mut(range)
            }
        }

        impl $crate::core::ops::Index<$crate::core::ops::RangeFrom<Size>> for $name {
            type Output = str;

            #[inline]
            fn index(&self, index: $crate::core::ops::RangeFrom<Size>) -> &Self::Output {
                let start = index.start as usize;
                self.as_str().index($crate::core::ops::RangeFrom { start })
            }
        }

        impl $crate::core::ops::Index<$crate::core::ops::RangeTo<Size>> for $name {
            type Output = str;

            #[inline]
            fn index(&self, index: $crate::core::ops::RangeTo<Size>) -> &Self::Output {
                let end = index.end as usize;
                self.as_str().index($crate::core::ops::RangeTo { end })
            }
        }

        impl $crate::core::ops::Index<$crate::core::ops::RangeFull> for $name {
            type Output = str;

            #[inline]
            fn index(&self, index: $crate::core::ops::RangeFull) -> &Self::Output {
                self.as_str().index(index)
            }
        }

        impl $crate::core::ops::Index<$crate::core::ops::Range<Size>> for $name {
            type Output = str;

            #[inline]
            fn index(&self, index: $crate::core::ops::Range<Size>) -> &Self::Output {
                let (start, end) = (index.start as usize, index.end as usize);
                self.as_str().index($crate::core::ops::Range { start, end })
            }
        }

        impl $crate::core::ops::Index<$crate::core::ops::RangeToInclusive<Size>> for $name {
            type Output = str;

            #[inline]
            fn index(&self, index: $crate::core::ops::RangeToInclusive<Size>) -> &Self::Output {
                let end = index.end as usize;
                self.as_str()
                    .index($crate::core::ops::RangeToInclusive { end })
            }
        }

        impl $crate::core::ops::Index<$crate::core::ops::RangeInclusive<Size>> for $name {
            type Output = str;

            #[inline]
            fn index(&self, index: $crate::core::ops::RangeInclusive<Size>) -> &Self::Output {
                let (start, end) = (*index.start() as usize, *index.end() as usize);
                let range = $crate::core::ops::RangeInclusive::new(start, end);
                self.as_str().index(range)
            }
        }

        #[cfg_attr(docs_rs_workaround, doc(cfg(feature = "serde-traits")))]
        #[cfg(feature = "serde-traits")]
        impl $crate::serde::Serialize for $name {
            #[inline]
            fn serialize<S: $crate::serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
                ser.serialize_str(self.as_str())
            }
        }

        #[cfg_attr(docs_rs_workaround, doc(cfg(feature = "serde-traits")))]
        #[cfg(feature = "serde-traits")]
        impl<'a> $crate::serde::Deserialize<'a> for $name {
            #[inline]
            fn deserialize<D: $crate::serde::Deserializer<'a>>(des: D) -> Result<Self, D::Error> {
                /// Abstracts deserializer visitor
                #[cfg(feature = "serde-traits")]
                struct InnerVisitor<'a>(pub $crate::core::marker::PhantomData<&'a ()>);

                #[cfg(feature = "serde-traits")]
                impl<'a> $crate::serde::de::Visitor<'a> for InnerVisitor<'a> {
                    type Value = $name;

                    #[inline]
                    fn expecting(
                        &self,
                        f: &mut $crate::core::fmt::Formatter,
                    ) -> $crate::core::fmt::Result {
                        write!(f, "a string")
                    }

                    #[inline]
                    fn visit_str<E: $crate::serde::de::Error>(
                        self,
                        v: &str,
                    ) -> Result<Self::Value, E> {
                        Ok($name::from_str_truncate(v))
                    }
                }

                des.deserialize_str(InnerVisitor($crate::core::marker::PhantomData))
            }
        }
    };
}
