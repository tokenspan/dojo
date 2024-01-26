mod async_graphql;

use std::error::Error;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;

use crate::order_by::Direction;
use crate::types::{accepts, to_sql_checked, IsNull, ToSql, Type};
use crate::Model;
use anyhow::Result;
use base64ct::{Base64, Encoding};
use bytes::BytesMut;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tracing::debug;
use tracing_subscriber::fmt::format;

pub use async_graphql::*;

pub trait DefaultSortKeys {
    fn keys() -> Vec<String>;

    fn order_by_stmt(direction: Direction) -> String {
        let mut stmt = "".to_string();
        for (i, order) in Self::keys().iter().enumerate() {
            if i > 0 {
                stmt.push_str(", ");
            }
            stmt.push_str(order);
            if i == 0 {
                if direction == Direction::Asc {
                    stmt.push_str(" ASC");
                } else {
                    stmt.push_str(" DESC");
                }
            } else {
                stmt.push_str(" ASC");
            }
        }

        stmt
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub column: String,
    pub value: crate::model::Value,
}

impl Row {
    pub fn new(column: String, value: crate::model::Value) -> Self {
        Self { column, value }
    }
}

#[derive(Debug, Clone)]
pub struct Cursor {
    pub values: Vec<Row>,
}

impl Cursor {
    pub fn to_where_stmt(
        &self,
        direction: Direction,
        params_index: &mut usize,
    ) -> (String, Vec<&(dyn ToSql + Sync)>) {
        let mut columns = vec![];
        let mut stmt = "".to_string();

        for value in &self.values {
            columns.push(value.column.clone());
        }

        stmt.push('(');
        stmt.push_str(&columns.join(", "));
        stmt.push_str(") ");
        let ch = if direction == Direction::Asc {
            '>'
        } else {
            '<'
        };
        stmt.push(ch);
        stmt.push_str(" (");

        let mut params: Vec<&(dyn ToSql + Sync)> = vec![];
        let mut args = vec![];
        for value in &self.values {
            params.push(&value.value);
            args.push(format!("${}", params_index));
            *params_index += 1;
        }

        stmt.push_str(&args.join(", "));
        stmt.push(')');

        (stmt, params)
    }
}

impl Cursor {
    pub fn new(values: Vec<Row>) -> Self {
        Self { values }
    }

    pub fn encode(&self) -> String {
        // it's safe, trust me bro.
        let buf = bincode::serialize(&self.values).unwrap();
        Base64::encode_string(&buf)
    }

    pub fn decode(encoded: &str) -> Result<Self> {
        let decoded = Base64::decode_vec(encoded).unwrap();
        let values: Vec<Row> = bincode::deserialize(&decoded[..])?;
        Ok(Self { values })
    }
}

#[derive(Debug)]
pub struct Pagination<T>
where
    T: Model + Debug,
{
    pub items: Vec<T>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
    pub first: Option<i64>,
    pub last: Option<i64>,
    pub total_nodes: i64,
    pub has_next: bool,
    pub has_previous: bool,
}

impl<T> Pagination<T>
where
    T: Model + Debug,
{
    pub fn new(
        items: Vec<T>,
        first: Option<i64>,
        after: Option<Cursor>,
        last: Option<i64>,
        before: Option<Cursor>,
        total_nodes: i64,
    ) -> Self {
        let limit = first.or(last).unwrap_or(0);
        let has_next_or_previous = items.len() as i64 == limit + 1;
        let has_next = first.is_some() && has_next_or_previous;
        let has_previous = last.is_some() && has_next_or_previous;

        let mut items = items;
        if has_next_or_previous {
            items.pop();
        }

        Self {
            items,
            before,
            after,
            first,
            last,
            total_nodes,
            has_next,
            has_previous,
        }
    }

    pub fn end_cursor(&self) -> Option<Cursor> {
        self.items.last().map(|item| item.cursor())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use dojo_macros::Model;
    use googletest::prelude::*;
    use uuid::Uuid;

    #[test]
    fn test_cursor_to_sql_with_1_key() -> anyhow::Result<()> {
        let created_at = NaiveDateTime::parse_from_str("2024-01-07 12:34:56", "%Y-%m-%d %H:%M:%S")?;
        let cursor_value = Row {
            column: "created_at".to_string(),
            value: crate::model::Value::NaiveDateTime(created_at),
        };
        let cursor = Cursor::new(vec![cursor_value]);
        let (sql, params) = cursor.to_where_stmt(Direction::Asc, &mut 1);
        println!("sql: {}", sql);
        println!("params: {:?}", params);

        Ok(())
    }

    #[test]
    fn test_cursor_to_sql_with_2_key() -> anyhow::Result<()> {
        let created_at = NaiveDateTime::parse_from_str("2024-01-07 12:34:56", "%Y-%m-%d %H:%M:%S")?;
        let uuid = Uuid::parse_str("ce2087a7-bdbc-4453-9fb8-d4dff3584f3e")?;
        let cursor = Cursor::new(vec![
            Row {
                column: "created_at".to_string(),
                value: crate::model::Value::NaiveDateTime(created_at),
            },
            Row {
                column: "id".to_string(),
                value: crate::model::Value::Uuid(uuid),
            },
        ]);
        let (sql, params) = cursor.to_where_stmt(Direction::Asc, &mut 1);
        println!("sql: {}", sql);
        println!("params: {:?}", params);

        Ok(())
    }

    #[test]
    fn test_decode_cursor() -> anyhow::Result<()> {
        let created_at = NaiveDateTime::parse_from_str("2024-01-07 12:34:56", "%Y-%m-%d %H:%M:%S")?;
        let cursor_value = Row {
            column: "created_at".to_string(),
            value: crate::model::Value::NaiveDateTime(created_at),
        };
        let cursor = Cursor::new(vec![cursor_value]);
        let encoded = cursor.encode();

        let decoded = Cursor::decode(&encoded).unwrap();
        assert_that!(
            decoded,
            pat!(Cursor {
                values: contains_each![pat!(Row {
                    column: eq("created_at".to_string()),
                    value: eq(crate::model::Value::NaiveDateTime(created_at)),
                }),],
            })
        );

        Ok(())
    }
}
