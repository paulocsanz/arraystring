use crate::prelude::*;
use generic_array::ArrayLength;

/*
#[cfg(feature = "diesel-traits")]
use core::io::Write;

#[cfg(feature = "diesel-traits")]
use diesel::{backend::Backend, sql_types::VarChar, Expression, row::Row};

#[cfg(feature = "diesel-traits")]
use diesel::deserialize::{self, FromSql, FromSqlRow, Queryable};

#[cfg(feature = "diesel-traits")]
use diesel::serialize::{self, Output, ToSql};
*/

#[cfg(feature = "serde-traits")]
use serde::{de, de::Deserializer, de::Visitor, ser::Serializer, Deserialize, Serialize};

#[cfg(feature = "serde-traits")]
use core::marker::PhantomData;

#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "serde-traits")))]
#[cfg(feature = "serde-traits")]
impl<SIZE> Serialize for ArrayString<SIZE>
where
    SIZE: ArrayLength<u8>,
{
    #[inline]
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}

#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "serde-traits")))]
#[cfg(feature = "serde-traits")]
impl<'a, SIZE> Deserialize<'a> for ArrayString<SIZE>
where
    SIZE: ArrayLength<u8>,
{
    #[inline]
    fn deserialize<D: Deserializer<'a>>(des: D) -> Result<Self, D::Error> {
        /// Abstracts deserializer visitor
        struct InnerVisitor<'a, SIZE: ArrayLength<u8>>(pub PhantomData<(&'a (), SIZE)>);

        impl<'a, SIZE: ArrayLength<u8>> Visitor<'a> for InnerVisitor<'a, SIZE> {
            type Value = ArrayString<SIZE>;

            #[inline]
            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, "a string")
            }

            #[inline]
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(ArrayString::from_str_truncate(v))
            }
        }

        des.deserialize_str(InnerVisitor(PhantomData))
    }
}

/*
#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "diesel-traits")))]
#[cfg(feature = "diesel-traits")]
impl<SIZE: ArrayLength<u8>> Expression for ArrayString<SIZE> {
    type SqlType = VarChar;
}

#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "diesel-traits")))]
#[cfg(feature = "diesel-traits")]
impl<SIZE, ST, DB> FromSql<ST, DB> for ArrayString<SIZE>
where
    SIZE: ArrayLength<u8>,
    DB: Backend,
    *const str: FromSql<ST, DB>,
{
    #[inline]
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        let ptr = <*const str as FromSql<ST, DB>>::from_sql(bytes)?;
        // We know that the pointer impl will never return null
        let string = unsafe { &*ptr };
        Ok(Self::from_str_truncate(string))
    }
}

#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "diesel-traits")))]
#[cfg(feature = "diesel-traits")]
impl<SIZE, ST, DB> FromSqlRow<ST, DB> for ArrayString<SIZE>
where
    SIZE: ArrayLength<u8>,
    DB: Backend,
    *const str: FromSql<ST, DB>,
{
    const FIELDS_NEEDED: usize = 1;

    #[inline]
    fn build_from_row<T: Row<DB>>(row: &mut T) -> deserialize::Result<Self> {
        <Self as FromSql<ST, DB>>::from_sql(row.take())
    }
}

#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "diesel-traits")))]
#[cfg(feature = "diesel-traits")]
impl<SIZE, ST, DB> Queryable<ST, DB> for ArrayString<SIZE>
where
    SIZE: ArrayLength<u8>,
    DB: Backend,
    *const str: FromSql<ST, DB>,
{
    type Row = Self;

    #[inline]
    fn build(row: Self::Row) -> Self {
        row
    }
}

#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "diesel-traits")))]
#[cfg(feature = "diesel-traits")]
impl<SIZE, DB> ToSql<VarChar, DB> for ArrayString<SIZE>
where
    SIZE: ArrayLength<u8>,
    DB: Backend,
{
    #[inline]
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        <str as ToSql<VarChar, DB>>::to_sql(self.as_str(), out)
    }
}

#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "diesel-traits")))]
#[cfg(feature = "diesel-traits")]
impl Expression for CacheString {
    type SqlType = VarChar;
}

#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "diesel-traits")))]
#[cfg(feature = "diesel-traits")]
impl<ST, DB> FromSql<ST, DB> for CacheString
where
    DB: Backend,
    *const str: FromSql<ST, DB>,
{
    #[inline]
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        ArrayString::from_sql(bytes).map(|a| CacheString(a))
    }
}

#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "diesel-traits")))]
#[cfg(feature = "diesel-traits")]
impl<DB> ToSql<VarChar, DB> for CacheString
where
    DB: Backend,
{
    #[inline]
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        self.0.to_sql(out)
    }
}
*/

#[cfg(test)]
mod tests {
    /*
    #![allow(proc_macro_derive_resolution_fallback)]
    #![allow(unused_import_braces)]
    */

    use super::*;
    use crate::ArrayString;

    #[cfg(feature = "serde-traits")]
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct DeriveSerde(pub ArrayString<generic_array::typenum::U8>);

    #[cfg(feature = "serde-traits")]
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Derive2Serde(pub CacheString);

    #[test]
    #[cfg(feature = "serde-traits")]
    fn derive_json() {
        let string =
            serde_json::to_string(&DeriveSerde(ArrayString::try_from_str("abcdefg").unwrap()))
                .unwrap();
        let s: DeriveSerde = serde_json::from_str(&string).unwrap();
        assert_eq!(
            s,
            DeriveSerde(ArrayString::try_from_str("abcdefg").unwrap())
        );
    }

    #[test]
    #[cfg(feature = "serde-traits")]
    fn derive2_json() {
        let string = serde_json::to_string(&Derive2Serde(CacheString(
            ArrayString::try_from_str("abcdefg").unwrap(),
        )))
        .unwrap();
        let s: DeriveSerde = serde_json::from_str(&string).unwrap();
        assert_eq!(
            s,
            DeriveSerde(ArrayString::try_from_str("abcdefg").unwrap())
        );
    }

    #[test]
    #[cfg(feature = "serde-traits")]
    fn json() {
        let string =
            serde_json::to_string(&ArrayString::<generic_array::typenum::U8>::try_from_str("abcdefg").unwrap())
                .unwrap();
        let s: ArrayString<generic_array::typenum::U8> = serde_json::from_str(&string).unwrap();
        assert_eq!(
            s,
            ArrayString::<generic_array::typenum::U8>::try_from_str("abcdefg").unwrap()
        );
    }

    /*
    #[cfg(feature = "diesel-traits")]
    use diesel::{insert_into, prelude::*, update, debug_query, pg, mysql, sqlite};

    #[cfg(feature = "diesel-traits")]
    #[macro_use]
    table! {
        derives (id) {
            id -> Int4,
            name -> VarChar,
        }
    }

    #[cfg(feature = "diesel-traits")]
    #[derive(Queryable, Insertable, Clone, Debug)]
    #[table_name = "derives"]
    struct DeriveDiesel {
        pub name: ArrayString<generic_array::typenum::U32>,
    }

    #[cfg(feature = "diesel-traits")]
    #[derive(Queryable, Insertable, Clone, Debug)]
    #[table_name = "derives"]
    struct Derive2Diesel {
        pub name: CacheString,
    }

    #[cfg(feature = "diesel-traits")]
    #[derive(Queryable, Insertable, Clone, Debug)]
    #[table_name = "derives"]
    struct Derive3Diesel {
        pub name: String,
    }

    #[cfg(feature = "diesel-traits")]
    fn derive_query_compare() {
        let array = DeriveDiesel {
            name: ArrayString::try_from_str("Name1").unwrap(),
        };
        let cache = Derive2Diesel {
            name: CacheString(ArrayString::try_from_str("Name1").unwrap()),
        };
        let string = Derive3Diesel {
            name: String::from("Name1"),
        };

        let insert_array = insert_into(derives::table).values(&array);
        let insert_cache = insert_into(derives::table).values(&array);
        let insert_string = insert_into(derives::table).values(&string);
        assert_eq!(debug_query::<pg::Pg, _>(&insert_array).to_string(), debug_query::<pg::Pg, _>(&insert_string).to_string());
        assert_eq!(debug_query::<pg::Pg, _>(&insert_cache).to_string(), debug_query::<pg::Pg, _>(&insert_string).to_string());
        assert_eq!(debug_query::<mysql::Mysql, _>(&insert_array).to_string(), debug_query::<mysql::Mysql, _>(&insert_string).to_string());
        assert_eq!(debug_query::<mysql::Mysql, _>(&insert_cache).to_string(), debug_query::<mysql::Mysql, _>(&insert_string).to_string());
        assert_eq!(debug_query::<sqlite::Sqlite, _>(&insert_array).to_string(), debug_query::<sqlite::Sqlite, _>(&insert_string).to_string());
        assert_eq!(debug_query::<sqlite::Sqlite, _>(&insert_cache).to_string(), debug_query::<sqlite::Sqlite, _>(&insert_string).to_string());

        let update_array = update(derives::table).set(derives::name.eq(&array.name));
        let update_cache = update(derives::table).set(derives::name.eq(&array.name));
        let update_string = update(derives::table).set(derives::name.eq(&string.name));
        assert_eq!(debug_query::<pg::Pg, _>(&update_array).to_string(), debug_query::<pg::Pg, _>(&update_string).to_string());
        assert_eq!(debug_query::<pg::Pg, _>(&update_cache).to_string(), debug_query::<pg::Pg, _>(&update_string).to_string());
        assert_eq!(debug_query::<mysql::Mysql, _>(&update_array).to_string(), debug_query::<mysql::Mysql, _>(&update_string).to_string());
        assert_eq!(debug_query::<mysql::Mysql, _>(&update_cache).to_string(), debug_query::<mysql::Mysql, _>(&update_string).to_string());
        assert_eq!(debug_query::<sqlite::Sqlite, _>(&update_array).to_string(), debug_query::<sqlite::Sqlite, _>(&update_string).to_string());
        assert_eq!(debug_query::<sqlite::Sqlite, _>(&update_cache).to_string(), debug_query::<sqlite::Sqlite, _>(&update_string).to_string());
    }

    #[cfg(feature = "diesel-traits")]
    fn derive_query_sqlite() {
        let conn = diesel::sqlite::SqliteConnection::establish(":memory:").unwrap();
        let string = DeriveDiesel {
            name: ArrayString::try_from_str("Name1").unwrap(),
        };

        let _ = insert_into(derives::table)
            .values(&string)
            .execute(&conn)
            .unwrap();

        let queried: DeriveDiesel = derives::table.first(&conn).unwrap();
        assert_eq!(queried.name.as_str(), "Name1");
    }

    #[cfg(feature = "diesel-traits")]
    fn derive2_query_sqlite() {
        let conn = diesel::sqlite::SqliteConnection::establish(":memory:").unwrap();
        let string = Derive2Diesel {
            name: CacheString(ArrayString::try_from_str("Name1").unwrap()),
        };

        let _ = insert_into(derives::table)
            .values(&string)
            .execute(&conn)
            .unwrap();

        let queried: Derive2Diesel = derives::table.first(&conn).unwrap();
        assert_eq!(queried.name.as_str(), "Name1");
    }
    */
}
