//! All macros related to this crate

/// Abstracts [`ArrayString`] implementation
///
/// [`ArrayString`]: ./traits/trait.ArrayString.html
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
        #[cfg_attr(features = "diesel-traits", derive(FromSqlRow, AsExpression, SqlType))]
        #[allow(trivial_numeric_casts)]
        $(#[$attr])*
        pub struct $name([u8; $size as usize], $crate::Size);
        __inner_impl_string!($name, $size);
    };
    ($(#[$attr:meta])* struct $name: ident ($size: expr)) => {
        /// Customized stack based string
        #[derive(Copy, Clone)]
        #[allow(trivial_numeric_casts)]
        #[cfg_attr(features = "diesel-traits", derive(FromSqlRow, AsExpression, SqlType))]
        $(#[$attr])*
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

        impl $name {
            /// Extracts interior byte slice (with used length)
            ///
            /// ```rust
            /// # use arraystring::{prelude::*, error::Error};
            /// # fn main() -> Result<(), Error> {
            /// let string = CacheString::try_from_str("Byte Slice")?;
            /// let (array, len) = string.into_bytes();
            /// assert_eq!(&array[..len as usize], b"Byte Slice");
            /// # Ok(())
            /// # }
            /// ```
            #[inline]
            #[allow(trivial_numeric_casts)]
            pub fn into_bytes(mut self) -> ([u8; $size as usize], Size) {
                use $crate::core::ptr::write_bytes;
                let dest = unsafe { (&mut self.0 as *mut [u8] as *mut u8).add(self.len().into()) };
                unsafe { write_bytes(dest, 0, Self::CAPACITY.saturating_sub(self.len()).into());
                }
                (self.0, self.1)
            }
        }

        impl $crate::traits::Buffer for $name {
            #[inline]
            unsafe fn buffer(&mut self) -> &mut [u8] {
                self.0.as_mut()
            }

            #[inline]
            fn update_len<F>(&mut self, f: F)
            where
                F: FnOnce(&mut Size)
            {
                f(&mut self.1)
            }

            #[inline]
            fn fetch_len(&self) -> Size {
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
                use $crate::traits::Buffer;
                let len = self.fetch_len() as usize;
                let slice = unsafe { self.0.get_unchecked_mut(..len) };
                unsafe { $crate::core::str::from_utf8_unchecked_mut(slice) }
            }
        }

        impl AsRef<[u8]> for $name {
            #[inline]
            fn as_ref(&self) -> &[u8] {
                use $crate::traits::Buffer;
                unsafe { self.0.get_unchecked(..self.fetch_len().into()) }
            }
        }

        impl $crate::ArrayString for $name {
            const CAPACITY: Size = $size;

            /*
            #[inline]
            unsafe fn from_str_unchecked<S>(string: S) -> Self
            where
                S: AsRef<str>,
            {
                use $crate::core::{mem::uninitialized, ptr::copy_nonoverlapping, ptr::write_bytes};
                debug_assert!(string.as_ref().len() <= Self::CAPACITY as usize);
                let mut array: [u8; Self::CAPACITY as usize] = uninitialized();
                let (s, dest) = (string.as_ref(), &mut array as *mut [u8] as *mut u8);
                copy_nonoverlapping(s.as_ptr(), dest, s.len());
                write_bytes(dest.add(s.len()), 0, (Self::CAPACITY as usize).saturating_sub(s.len()));
                $name(array, s.len() as Size)
            }

            #[inline]
            unsafe fn from_iterator_unchecked<U, I>(iter: I) -> Self
            where
                U: AsRef<str>,
                I: IntoIterator<Item = U>,
            {
                use $crate::core::{mem::uninitialized, ptr::copy_nonoverlapping, ptr::write_bytes};
                let mut array: [u8; Self::CAPACITY as usize] = uninitialized();
                let mut dest = &mut array as *mut [u8] as *mut u8;
                let mut size: usize = 0;

                for s in iter {
                    let s = s.as_ref();
                    copy_nonoverlapping(s.as_ptr(), dest, s.len());
                    size = size.saturating_add(s.len());
                    dest = dest.add(s.len());
                }
                write_bytes(dest, 0, (Self::CAPACITY as usize).saturating_sub(size));
                $name(array, size as Size)
            }
            */
        }

        impl $crate::core::str::FromStr for $name {
            type Err = $crate::error::OutOfBounds;

            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::try_from_str(s)
            }
        }

        impl $crate::core::fmt::Debug for $name {
            #[inline]
            fn fmt(&self, f: &mut $crate::core::fmt::Formatter) -> $crate::core::fmt::Result {
                use $crate::traits::Buffer;
                let s: &str = self.as_ref();
                f.debug_tuple(stringify!($name))
                    .field(&s)
                    .field(&self.fetch_len())
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
                self.as_mut_str().index_mut(start)
            }
        }

        impl $crate::core::ops::IndexMut<$crate::core::ops::RangeTo<Size>> for $name {
            #[inline]
            fn index_mut(&mut self, index: $crate::core::ops::RangeTo<Size>) -> &mut str {
                let end = index.end as usize;
                self.as_mut_str()
                    .index_mut($crate::core::ops::RangeTo { end })
            }
        }

        impl $crate::core::ops::IndexMut<$crate::core::ops::RangeFull> for $name {
            #[inline]
            fn index_mut(&mut self, index: $crate::core::ops::RangeFull) -> &mut str {
                self.as_mut_str().index_mut(index)
            }
        }

        impl $crate::core::ops::IndexMut<$crate::core::ops::Range<Size>> for $name {
            #[inline]
            fn index_mut(&mut self, index: $crate::core::ops::Range<Size>) -> &mut str {
                let (start, end) = (index.start as usize, index.end as usize);
                let range = $crate::core::ops::Range { start, end };
                self.as_mut_str().index_mut(range)
            }
        }

        impl $crate::core::ops::IndexMut<$crate::core::ops::RangeToInclusive<Size>> for $name {
            #[inline]
            fn index_mut(&mut self, index: $crate::core::ops::RangeToInclusive<Size>) -> &mut str {
                let end = index.end as usize;
                let range = $crate::core::ops::RangeToInclusive { end };
                self.as_mut_str().index_mut(range)
            }
        }

        impl $crate::core::ops::IndexMut<$crate::core::ops::RangeInclusive<Size>> for $name {
            #[inline]
            fn index_mut(&mut self, index: $crate::core::ops::RangeInclusive<Size>) -> &mut str {
                let (start, end) = (*index.start() as usize, *index.end() as usize);
                let range = $crate::core::ops::RangeInclusive::new(start, end);
                self.as_mut_str().index_mut(range)
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

        #[cfg_attr(docs_rs_workaround, doc(cfg(feature = "diesel-traits")))]
        #[cfg(feature = "diesel-traits")]
        impl $crate::diesel::expression::Expression for $name {
            type SqlType = $crate::diesel::sql_types::VarChar;
        }


        #[cfg_attr(docs_rs_workaround, doc(cfg(feature = "diesel-traits")))]
        #[cfg(feature = "diesel-traits")]
        impl<ST, DB> $crate::diesel::deserialize::FromSql<ST, DB> for $name
        where
            DB: $crate::diesel::backend::Backend,
            *const str: $crate::diesel::deserialize::FromSql<ST, DB>
        {
            #[inline]
            fn from_sql(bytes: Option<&DB::RawValue>) -> $crate::diesel::deserialize::Result<Self> {
                let ptr = <*const str as $crate::diesel::deserialize::FromSql<ST, DB>>::from_sql(bytes)?;
                // We know that the pointer impl will never return null
                let string = unsafe { &*ptr };
                Ok(Self::from_str_truncate(string))
            }
        }

        #[cfg_attr(docs_rs_workaround, doc(cfg(feature = "diesel-traits")))]
        #[cfg(feature = "diesel-traits")]
        impl<DB> $crate::diesel::serialize::ToSql<$crate::diesel::sql_types::VarChar, DB> for $name
        where
            DB: $crate::diesel::backend::Backend,
        {
            #[inline]
            fn to_sql<W: $crate::core::io::Write>(&self, out: &mut $crate::diesel::serialize::Output<W, DB>) -> $crate::diesel::serialize::Result {
                <str as $crate::diesel::serialize::ToSql<$crate::diesel::sql_types::VarChar, DB>>::to_sql(self.as_str(), out)
            }
        }
    };
}
