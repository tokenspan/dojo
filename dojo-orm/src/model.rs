use crate::pagination::{Cursor, Row};
use crate::types::ToSql;
use anyhow::Result;
use async_graphql::Enum;
use chrono::NaiveDateTime;
use postgres_types::{accepts, to_sql_checked};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum::{Display, EnumString};
use uuid::Uuid;

macro_rules! impl_value {
    ($ty: ty, $variant: ident) => {
        impl From<$ty> for Value {
            fn from(value: $ty) -> Self {
                Value::$variant(value)
            }
        }

        impl From<&$ty> for Value {
            fn from(value: &$ty) -> Self {
                Value::$variant(value.clone())
            }
        }

        impl From<Option<$ty>> for Value {
            fn from(_: Option<$ty>) -> Self {
                Value::Null
            }
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Value {
    Uuid(Uuid),
    Int32(i32),
    Int64(i64),
    String(String),
    NaiveDateTime(NaiveDateTime),
    Null,
}

impl_value!(Uuid, Uuid);
impl_value!(i32, Int32);
impl_value!(i64, Int64);
impl_value!(String, String);
impl_value!(NaiveDateTime, NaiveDateTime);

impl ToSql for Value {
    fn to_sql(
        &self,
        ty: &crate::types::Type,
        w: &mut bytes::BytesMut,
    ) -> std::result::Result<crate::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        match self {
            Value::Uuid(t) => t.to_sql(ty, w),
            Value::Int32(t) => t.to_sql(ty, w),
            Value::Int64(t) => t.to_sql(ty, w),
            Value::String(t) => t.to_sql(ty, w),
            Value::NaiveDateTime(t) => t.to_sql(ty, w),
            Value::Null => Ok(crate::types::IsNull::Yes),
        }
    }

    accepts!(UUID, INT4, INT8, TEXT, TIMESTAMP);
    to_sql_checked!();
}

pub trait Model {
    const NAME: &'static str;
    const COLUMNS: &'static [&'static str];
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
    fn from_row(row: tokio_postgres::Row) -> Result<Self>
    where
        Self: Sized;

    fn get_value(&self, column: &str) -> Option<Value>;
    fn sort_keys() -> Vec<String>;
    fn cursor(&self) -> Cursor {
        let mut values = vec![];
        for key in Self::sort_keys() {
            if let Some(value) = self.get_value(key.as_str()) {
                values.push(Row::new(key, value));
            }
        }

        Cursor::new(values)
    }
}

pub trait UpdateModel {
    const COLUMNS: &'static [&'static str];
    fn columns(&self) -> Vec<&'static str>;
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}
