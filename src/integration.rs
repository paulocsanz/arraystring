use generic_array::ArrayLength;
use crate::prelude::*;

#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "serde-traits")))]
#[cfg(feature = "serde-traits")]
impl<SIZE: ArrayLength<u8>> serde::Serialize for ArrayString<SIZE> {
    #[inline]
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}

#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "serde-traits")))]
#[cfg(feature = "serde-traits")]
impl<'a, SIZE: ArrayLength<u8>> serde::Deserialize<'a> for ArrayString<SIZE> {
    #[inline]
    fn deserialize<D: serde::Deserializer<'a>>(des: D) -> Result<Self, D::Error> {
        use core::marker::PhantomData;

        /// Abstracts deserializer visitor
        struct InnerVisitor<'a, SIZE: ArrayLength<u8>>(pub PhantomData<(&'a (), SIZE)>);

        impl<'a, SIZE: ArrayLength<u8>> serde::de::Visitor<'a> for InnerVisitor<'a, SIZE> {
            type Value = ArrayString<SIZE>;

            #[inline]
            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, "a string")
            }

            #[inline]
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(ArrayString::from_str_truncate(v))
            }
        }

        des.deserialize_str(InnerVisitor(PhantomData))
    }
}

#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "diesel-traits")))]
#[cfg(feature = "diesel-traits")]
impl<SIZE: ArrayLength<u8>> diesel::expression::Expression for ArrayString<SIZE> {
    type SqlType = diesel::sql_types::VarChar;
}

#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "diesel-traits")))]
#[cfg(feature = "diesel-traits")]
impl<SIZE: ArrayLength<u8>, ST, DB> diesel::deserialize::FromSql<ST, DB> for ArrayString<SIZE>
where
    DB: diesel::backend::Backend,
    *const str: diesel::deserialize::FromSql<ST, DB>,
{
    #[inline]
    fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
        let ptr = <*const str as diesel::deserialize::FromSql<ST, DB>>::from_sql(bytes)?;
        // We know that the pointer impl will never return null
        let string = unsafe { &*ptr };
        Ok(Self::from_str_truncate(string))
    }
}

#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "diesel-traits")))]
#[cfg(feature = "diesel-traits")]
impl<SIZE: ArrayLength<u8>, DB> diesel::serialize::ToSql<diesel::sql_types::VarChar, DB>
    for ArrayString<SIZE>
where
    DB: diesel::backend::Backend,
{
    #[inline]
    fn to_sql<W: core::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, DB>,
    ) -> diesel::serialize::Result {
        <str as diesel::serialize::ToSql<diesel::sql_types::VarChar, DB>>::to_sql(
            self.as_str(),
            out,
        )
    }
}
