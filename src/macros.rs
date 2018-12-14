/// Abstracts [`StringHandler`] implementation
///
/// [`StringHandler`]: ./handler/trait.StringHandler.html
///
/// ```rust
/// # #[macro_use]
/// # extern crate limited_string;
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
        impl $crate::handler::RawStringHandler for $name {
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
                unsafe { ::std::str::from_utf8_unchecked(self.as_ref()) }
            }
        }

        impl AsMut<str> for $name {
            #[inline]
            fn as_mut(&mut self) -> &mut str {
                use $crate::handler::RawStringHandler;
                let len = self.get_len() as usize;
                let slice = unsafe { self.0.get_unchecked_mut(..len) };
                unsafe { ::std::str::from_utf8_unchecked_mut(slice) }
            }
        }

        impl AsRef<[u8]> for $name {
            #[inline]
            fn as_ref(&self) -> &[u8] {
                use $crate::handler::RawStringHandler;
                unsafe { self.0.get_unchecked(..self.get_len() as usize) }
            }
        }

        impl $crate::StringHandler for $name {
            const SIZE: $crate::Size = $size;
        }

        /// Creates new `StringHandler` from string slice if length is lower or equal than `SIZE`, otherwise returns a error.
        ///
        /// ```rust
        /// # use std::str::FromStr;
        /// # use limited_string::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// let string = LimitedString::from_str("My String")?;
        /// assert_eq!(string.as_str(), "My String");
        ///
        /// assert_eq!(LimitedString::from_str("")?.as_str(), "");
        ///
        /// let out_of_bounds = "0".repeat(LimitedString::SIZE as usize + 1);
        /// assert!(LimitedString::from_str(&out_of_bounds).is_err());
        /// # Ok(())
        /// # }
        /// ```
        impl ::std::str::FromStr for $name {
            type Err = $crate::OutOfBoundsError;

            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $crate::utils::from_str(s)
            }
        }

        impl ::std::fmt::Debug for $name {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                use $crate::handler::RawStringHandler;
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

        impl ::std::borrow::Borrow<str> for $name {
            #[inline]
            fn borrow(&self) -> &str {
                self.as_str()
            }
        }

        impl ::std::hash::Hash for $name {
            #[inline]
            fn hash<H: ::std::hash::Hasher>(&self, hasher: &mut H) {
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
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                self.as_str().cmp(other.as_str())
            }
        }

        impl PartialOrd for $name {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl<'a> ::std::ops::Add<&'a str> for $name {
            type Output = Self;

            #[inline]
            fn add(self, other: &'a str) -> Self::Output {
                let mut out = Self::default();
                unsafe { out.push_str_unchecked(self.as_str()) };
                out.push_str_truncate(other);
                out
            }
        }

        impl ::std::fmt::Write for $name {
            #[inline]
            fn write_str(&mut self, slice: &str) -> ::std::fmt::Result {
                self.push_str(slice).map_err(|_| ::std::fmt::Error)
            }
        }

        impl ::std::fmt::Display for $name {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl ::std::ops::Deref for $name {
            type Target = str;

            #[inline]
            fn deref(&self) -> &Self::Target {
                self.as_ref()
            }
        }

        impl ::std::ops::DerefMut for $name {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.as_mut()
            }
        }

        impl ::std::ops::IndexMut<::std::ops::RangeFrom<Size>> for $name {
            #[inline]
            fn index_mut(&mut self, index: ::std::ops::RangeFrom<Size>) -> &mut str {
                let start = index.start as usize;
                let start = ::std::ops::RangeFrom { start };
                self.as_str_mut().index_mut(start)
            }
        }

        impl ::std::ops::IndexMut<::std::ops::RangeTo<Size>> for $name {
            #[inline]
            fn index_mut(&mut self, index: ::std::ops::RangeTo<Size>) -> &mut str {
                let end = index.end as usize;
                self.as_str_mut().index_mut(::std::ops::RangeTo { end })
            }
        }

        impl ::std::ops::IndexMut<::std::ops::RangeFull> for $name {
            #[inline]
            fn index_mut(&mut self, index: ::std::ops::RangeFull) -> &mut str {
                self.as_str_mut().index_mut(index)
            }
        }

        impl ::std::ops::IndexMut<::std::ops::Range<Size>> for $name {
            #[inline]
            fn index_mut(&mut self, index: ::std::ops::Range<Size>) -> &mut str {
                let (start, end) = (index.start as usize, index.end as usize);
                let range = ::std::ops::Range { start, end };
                self.as_str_mut().index_mut(range)
            }
        }

        impl ::std::ops::IndexMut<::std::ops::RangeToInclusive<Size>> for $name {
            #[inline]
            fn index_mut(&mut self, index: ::std::ops::RangeToInclusive<Size>) -> &mut str {
                let end = index.end as usize;
                let range = ::std::ops::RangeToInclusive { end };
                self.as_str_mut().index_mut(range)
            }
        }

        impl ::std::ops::IndexMut<::std::ops::RangeInclusive<Size>> for $name {
            #[inline]
            fn index_mut(&mut self, index: ::std::ops::RangeInclusive<Size>) -> &mut str {
                let (start, end) = (*index.start() as usize, *index.end() as usize);
                let range = ::std::ops::RangeInclusive::new(start, end);
                self.as_str_mut().index_mut(range)
            }
        }

        impl ::std::ops::Index<::std::ops::RangeFrom<Size>> for $name {
            type Output = str;

            #[inline]
            fn index(&self, index: ::std::ops::RangeFrom<Size>) -> &Self::Output {
                let start = index.start as usize;
                self.as_str().index(::std::ops::RangeFrom { start })
            }
        }

        impl ::std::ops::Index<::std::ops::RangeTo<Size>> for $name {
            type Output = str;

            #[inline]
            fn index(&self, index: ::std::ops::RangeTo<Size>) -> &Self::Output {
                let end = index.end as usize;
                self.as_str().index(::std::ops::RangeTo { end })
            }
        }

        impl ::std::ops::Index<::std::ops::RangeFull> for $name {
            type Output = str;

            #[inline]
            fn index(&self, index: ::std::ops::RangeFull) -> &Self::Output {
                self.as_str().index(index)
            }
        }

        impl ::std::ops::Index<::std::ops::Range<Size>> for $name {
            type Output = str;

            #[inline]
            fn index(&self, index: ::std::ops::Range<Size>) -> &Self::Output {
                let (start, end) = (index.start as usize, index.end as usize);
                self.as_str().index(::std::ops::Range { start, end })
            }
        }

        impl ::std::ops::Index<::std::ops::RangeToInclusive<Size>> for $name {
            type Output = str;

            #[inline]
            fn index(&self, index: ::std::ops::RangeToInclusive<Size>) -> &Self::Output {
                let end = index.end as usize;
                self.as_str().index(::std::ops::RangeToInclusive { end })
            }
        }

        impl ::std::ops::Index<::std::ops::RangeInclusive<Size>> for $name {
            type Output = str;

            #[inline]
            fn index(&self, index: ::std::ops::RangeInclusive<Size>) -> &Self::Output {
                let (start, end) = (*index.start() as usize, *index.end() as usize);
                let range = ::std::ops::RangeInclusive::new(start, end);
                self.as_str().index(range)
            }
        }

        /*
        #[cfg(features = "serde-traits")]
        use serde_lib::{de::SeqAccess, de::Visitor, ser::SerializeSeq};
        #[cfg(features = "serde-traits")]
        use serde_lib::{Deserialize, Deserializer, Serialize, Serializer};
        #[cfg(features = "serde-traits")]
        use std::{fmt, fmt::Formatter, marker::PhantomData};

        /// Abstracts serializer visitor
        #[cfg(features = "serde-traits")]
        struct InnerVisitor<'a, T: 'a + Deserialize<'a>>(pub PhantomData<&'a T>);

        #[cfg(features = "serde-traits")]
        impl<'a> Visitor<'a> for InnerVisitor<'a, T> {
            type Value = $name;

            #[inline]
            fn expecting(&self, f: &mut Formatter) -> fmt::Result {
                write!(f, "a string")
            }

            #[inline]
            fn visit_seq<A: SeqAccess<'a>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let inner = $name::default();
                while let Some(value) = seq.next_element()? {
                    inner.append(value);
                }
                Ok(inner)
            }
        }

        #[cfg_attr(docs_rs_workaround, doc(cfg(feature = "serde-traits")))]
        impl<'a, T: 'a + Deserialize<'a>> Deserialize<'a> for Inner<T> {
            #[inline]
            fn deserialize<D: Deserializer<'a>>(des: D) -> Result<Self, D::Error> {
                debug!("Deserialize Inner");
                des.deserialize_seq(InnerVisitor(PhantomData))
            }
        }

        #[cfg_attr(docs_rs_workaround, doc(cfg(feature = "serde-traits")))]
        impl<T: Serialize> Serialize for VoluntaryServitude<T> {
            #[inline]
            fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
                trace!("Serialize VoluntaryServitude");
                let len = self.len();
                let mut sequence = ser.serialize_seq(Some(len))?;
                for (el, _) in self.iter().zip(0..len) {
                    sequence.serialize_element(el)?;
                }
                sequence.end()
            }
        }

        #[cfg_attr(docs_rs_workaround, doc(cfg(feature = "serde-traits")))]
        impl<'a, T: 'a + Deserialize<'a>> Deserialize<'a> for VoluntaryServitude<T> {
            #[inline]
            fn deserialize<D: Deserializer<'a>>(des: D) -> Result<Self, D::Error> {
                Inner::deserialize(des).map(Self::new)
            }
        }
        */
    };
}
