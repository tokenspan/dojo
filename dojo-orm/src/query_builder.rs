use futures_util::TryFutureExt;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

use crate::model::Model;
use crate::order_by::{Direction, OrderBy, OrderPredicate};
use crate::pagination::{Cursor, DefaultSortKeys, Pagination, Row};
use crate::pool::*;
use crate::predicates::{Expr, ExprValueType, Predicate};
use crate::types::ToSql;
use typed_builder::TypedBuilder;

#[derive(Debug, Eq, PartialEq)]
pub enum QueryType {
    Select,
    Paging,
    Delete,
    Insert,
    Update,
}

#[derive(TypedBuilder, Debug)]
pub struct QueryBuilder<'a> {
    pub table_name: &'a str,
    #[builder(default = & [])]
    pub columns: &'a [&'a str],
    #[builder(default = Vec::new())]
    pub default_keys: Vec<String>,
    #[builder(default = & [])]
    pub params: &'a [&'a (dyn ToSql + Sync)],
    #[builder(default = & [])]
    pub predicates: &'a [Predicate<'a>],
    #[builder(default = & [])]
    pub order_by: &'a [OrderPredicate<'a>],
    #[builder(default = & None)]
    pub before: &'a Option<Cursor>,
    #[builder(default = & None)]
    pub after: &'a Option<Cursor>,
    #[builder(default = None)]
    pub first: Option<i64>,
    #[builder(default = None)]
    pub last: Option<i64>,
    #[builder(default = true)]
    pub is_returning: bool,
    #[builder(default = & [])]
    pub returning: &'a [&'a str],
    #[builder(default = None, setter(strip_option))]
    pub limit: Option<i64>,
    pub ty: QueryType,
}

impl<'a> QueryBuilder<'a> {
    pub fn build_limit_sql(&self) -> String {
        let mut stmt = " LIMIT ".to_string();
        let limit = if self.ty == QueryType::Select {
            self.limit.unwrap_or(20)
        } else {
            self.first.or(self.last).unwrap_or(20) + 1
        };

        stmt.push_str(&format!("{}", limit));
        stmt
    }

    pub fn build_order_by_sql(&self) -> String {
        let mut stmt = "".to_string();
        if self.ty == QueryType::Select {
            let order_sql = OrderBy::new(self.order_by).to_sql();
            if !order_sql.is_empty() {
                stmt.push_str(" ORDER BY ");
                stmt.push_str(&order_sql);
            }
            return stmt;
        }

        let direction = if self.first.is_some() {
            Direction::Asc
        } else {
            Direction::Desc
        };

        let order_sql = if let Some(cursor) = self.before {
            cursor.to_order_by_stmt(direction)
        } else if let Some(cursor) = self.after {
            cursor.to_order_by_stmt(direction)
        } else {
            Cursor::order_by_stmt_by_keys(&self.default_keys, direction)
        };

        if !order_sql.is_empty() {
            stmt.push_str(" ORDER BY ");
            stmt.push_str(&order_sql);
        }

        stmt
    }

    pub fn build_where_sql(&self, params_index: &mut usize) -> (String, Vec<&(dyn ToSql + Sync)>) {
        let mut params = self.params.to_vec();
        let mut stmt = "".to_string();
        let mut predicates_str = vec![];

        if let Some(before) = self.before {
            let (before_sql, before_params) = before.to_where_stmt(Direction::Desc);
            predicates_str.push(before_sql);
            *params_index += before_params.len();
            params.extend(before_params);
        } else if let Some(after) = self.after {
            let (after_sql, after_params) = after.to_where_stmt(Direction::Asc);
            predicates_str.push(after_sql);
            *params_index += after_params.len();
            params.extend(after_params);
        }

        for predicate in self.predicates {
            let (predicate_sql, predicate_params) = predicate.to_sql(params_index);
            if let Some(predicate_sql) = predicate_sql {
                predicates_str.push(predicate_sql);
                params.extend(predicate_params);
            }
        }
        if !predicates_str.is_empty() {
            stmt.push_str(" WHERE ");
            stmt.push_str(&predicates_str.join(" AND "));
            // stmt.push_str(" ");
        }

        (stmt, params)
    }

    pub fn build_select_from_sql(&self) -> String {
        let mut stmt = "SELECT ".to_string();
        stmt.push_str(&self.columns.join(", "));
        stmt.push_str(" FROM ");
        stmt.push_str(self.table_name);

        stmt
    }

    pub fn build_delete_from_sql(&self) -> String {
        let mut stmt = "DELETE FROM ".to_string();
        stmt.push_str(self.table_name);

        stmt
    }

    pub fn build_update_from_sql(&self) -> String {
        let mut stmt = "UPDATE ".to_string();
        stmt.push_str(self.table_name);

        stmt
    }

    pub fn build_update_set_sql(&self, params_index: &mut usize) -> String {
        let mut stmt = " SET ".to_string();
        let mut sets = vec![];
        for column in self.columns {
            sets.push(format!("{} = ${}", column, params_index));
            *params_index += 1;
        }
        stmt.push_str(&sets.join(", "));

        stmt
    }

    pub fn build_select_sql(&self) -> (String, Vec<&(dyn ToSql + Sync)>) {
        let mut params_index = 1;
        let mut stmt = self.build_select_from_sql();

        let (where_sql, params) = self.build_where_sql(&mut params_index);
        stmt.push_str(&where_sql);

        let order_by_sql = self.build_order_by_sql();
        stmt.push_str(&order_by_sql);

        let limit_sql = self.build_limit_sql();
        stmt.push_str(&limit_sql);

        (stmt, params)
    }

    pub fn build_delete_sql(&self) -> (String, Vec<&(dyn ToSql + Sync)>) {
        let mut params_index = 1;
        let mut stmt = self.build_delete_from_sql();

        let (where_sql, params) = self.build_where_sql(&mut params_index);
        stmt.push_str(&where_sql);

        let returning_sql = self.build_returning_sql();
        stmt.push_str(&returning_sql);

        (stmt, params)
    }

    pub fn build_update_sql(&self) -> (String, Vec<&(dyn ToSql + Sync)>) {
        let mut params_index = 1;
        let mut stmt = self.build_update_from_sql();

        let set_sql = self.build_update_set_sql(&mut params_index);
        stmt.push_str(&set_sql);

        let (where_sql, params) = self.build_where_sql(&mut params_index);
        stmt.push_str(&where_sql);

        let returning_sql = self.build_returning_sql();
        stmt.push_str(&returning_sql);

        (stmt, params)
    }

    pub fn build_insert_sql(&self) -> (String, Vec<&(dyn ToSql + Sync)>) {
        let mut stmt = "INSERT INTO ".to_string();
        stmt.push_str(self.table_name);

        let mut columns = vec![];
        for column in self.columns {
            columns.push(column.to_string());
        }
        stmt.push_str(&format!(" ({}) VALUES ", columns.join(", ")));

        let chunks = self.params.chunks(self.columns.len()).collect::<Vec<_>>();
        let mut params_index = 1;
        let mut values = vec![];
        for chunk in chunks {
            let mut values_str = vec![];
            for _ in chunk {
                values_str.push(format!("${}", params_index));
                params_index += 1;
            }
            values.push(format!("({})", values_str.join(", ")));
        }
        stmt.push_str(&values.join(", "));

        let returning_sql = self.build_returning_sql();
        stmt.push_str(&returning_sql);

        (stmt, self.params.to_vec())
    }

    pub fn build_returning_sql(&self) -> String {
        let mut stmt = "".to_string();
        if self.is_returning {
            stmt.push_str(" RETURNING ");
            if self.returning.is_empty() {
                stmt.push_str(&self.columns.join(", "));
            } else {
                stmt.push_str(&self.returning.join(", "));
            }
        }

        stmt
    }

    pub fn build_sql(&self) -> anyhow::Result<(String, Vec<&(dyn ToSql + Sync)>)> {
        if self.first.is_some() && self.last.is_some() {
            return Err(anyhow::anyhow!(
                "first and last cannot be specified at the same time"
            ));
        }

        if self.after.is_some() && self.before.is_some() {
            return Err(anyhow::anyhow!(
                "after and before cannot be specified at the same time"
            ));
        }

        // if self.first.is_some() && self.after.is_none() {
        //     return Err(anyhow::anyhow!("first must be specified with after"));
        // }

        // if self.last.is_some() && self.before.is_none() {
        //     return Err(anyhow::anyhow!("last must be specified with before"));
        // }

        let (stmt, params) = match self.ty {
            QueryType::Select => self.build_select_sql(),
            QueryType::Paging => self.build_select_sql(),
            QueryType::Delete => self.build_delete_sql(),
            QueryType::Insert => self.build_insert_sql(),
            QueryType::Update => self.build_update_sql(),
        };

        Ok((stmt, params))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicates::equals;
    use crate::Value;
    use chrono::{NaiveDateTime, Utc};
    use dojo_macros::Model;
    use rstest::*;
    use uuid::Uuid;

    fn cursor() -> Cursor {
        Cursor {
            values: vec![
                Row::new(
                    "created_at".to_string(),
                    Value::NaiveDateTime(Utc::now().naive_utc()),
                ),
                Row::new(
                    "id".to_string(),
                    Value::Uuid(Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()),
                ),
            ],
        }
    }

    #[tokio::test]
    async fn test_build_default_query() -> anyhow::Result<()> {
        let qb = QueryBuilder::builder()
            .table_name("users")
            .columns(&["id", "name", "age", "created_at"])
            .default_keys(vec!["created_at".to_string(), "id".to_string()])
            .ty(QueryType::Paging)
            .build();

        let (stmt, params) = qb.build_sql()?;
        assert_eq!(
            stmt,
            "SELECT id, name, age, created_at FROM users ORDER BY created_at DESC, id ASC LIMIT 21"
        );
        assert_eq!(params.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_build_first_after_query() -> anyhow::Result<()> {
        let cursor = Some(cursor());
        let qb = QueryBuilder::builder()
            .table_name("users")
            .columns(&["id", "name", "age", "created_at"])
            .default_keys(vec!["created_at".to_string(), "id".to_string()])
            .first(Some(10))
            .after(&cursor)
            .ty(QueryType::Paging)
            .build();

        let (stmt, params) = qb.build_sql()?;
        assert_eq!(
            stmt,
            "SELECT id, name, age, created_at FROM users WHERE (created_at, id) > ($1, $2) ORDER BY created_at ASC, id ASC LIMIT 11"
        );
        assert_eq!(params.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_build_first_after_with_additional_predicates_query() -> anyhow::Result<()> {
        let cursor = Some(cursor());
        let predicates = &[equals("name", &"test"), equals("age", &20)];
        let qb = QueryBuilder::builder()
            .table_name("users")
            .columns(&["id", "name", "age", "created_at"])
            .default_keys(vec!["created_at".to_string(), "id".to_string()])
            .predicates(predicates)
            .first(Some(10))
            .after(&cursor)
            .ty(QueryType::Paging)
            .build();

        let (stmt, params) = qb.build_sql()?;
        assert_eq!(
            stmt,
            "SELECT id, name, age, created_at FROM users WHERE (created_at, id) > ($1, $2) AND name = $3 AND age = $4 ORDER BY created_at ASC, id ASC LIMIT 11"
        );
        assert_eq!(params.len(), 4);

        Ok(())
    }

    #[tokio::test]
    async fn test_build_last_before_query() -> anyhow::Result<()> {
        let cursor = Some(cursor());
        let qb = QueryBuilder::builder()
            .table_name("users")
            .columns(&["id", "name", "age", "created_at"])
            .default_keys(vec!["created_at".to_string(), "id".to_string()])
            .last(Some(10))
            .before(&cursor)
            .ty(QueryType::Paging)
            .build();
        let (stmt, params) = qb.build_sql()?;
        assert_eq!(
            stmt,
            "SELECT id, name, age, created_at FROM users WHERE (created_at, id) < ($1, $2) ORDER BY created_at DESC, id ASC LIMIT 11"
        );
        assert_eq!(params.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_build_last_with_additional_predicates_query() -> anyhow::Result<()> {
        let cursor = Some(cursor());
        let predicates = &[equals("name", &"test"), equals("age", &20)];
        let qb = QueryBuilder::builder()
            .table_name("users")
            .columns(&["id", "name", "age", "created_at"])
            .default_keys(vec!["created_at".to_string(), "id".to_string()])
            .predicates(predicates)
            .last(Some(10))
            .before(&cursor)
            .ty(QueryType::Paging)
            .build();
        let (stmt, params) = qb.build_sql()?;
        assert_eq!(
            stmt,
            "SELECT id, name, age, created_at FROM users WHERE (created_at, id) < ($1, $2) AND name = $3 AND age = $4 ORDER BY created_at DESC, id ASC LIMIT 11"
        );
        assert_eq!(params.len(), 4);

        Ok(())
    }

    #[tokio::test]
    async fn test_build_where_by_id() -> anyhow::Result<()> {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        let columns = &["id", "name", "age", "created_at"];
        let predicates = &[equals("id", &id)];
        let qb = QueryBuilder::builder()
            .table_name("users")
            .columns(columns)
            .default_keys(vec!["created_at".to_string(), "id".to_string()])
            .predicates(predicates)
            .ty(QueryType::Select)
            .build();
        let (stmt, params) = qb.build_sql()?;
        assert_eq!(
            stmt,
            "SELECT id, name, age, created_at FROM users WHERE id = $1 LIMIT 20"
        );
        assert_eq!(params.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_build_update_sql() -> anyhow::Result<()> {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        let name = "test".to_string();
        let params: Vec<&(dyn ToSql + Sync)> = vec![&name, &20];
        let predicates = &[equals("id", &id)];

        let qb = QueryBuilder::builder()
            .table_name("users")
            .columns(&["name", "age"])
            .default_keys(vec!["created_at".to_string(), "id".to_string()])
            .predicates(predicates)
            .params(params.as_slice())
            .ty(QueryType::Update)
            .is_returning(true)
            .returning(&["id", "name", "age", "created_at"])
            .build();
        let (stmt, params) = qb.build_sql()?;
        assert_eq!(stmt, "UPDATE users SET name = $1, age = $2 WHERE id = $3 RETURNING id, name, age, created_at");
        assert_eq!(params.len(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_build_insert_sql() -> anyhow::Result<()> {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        let name = "test".to_string();
        let created_at = NaiveDateTime::parse_from_str("2024-01-07 12:34:56", "%Y-%m-%d %H:%M:%S")?;
        let params: Vec<&(dyn ToSql + Sync)> = vec![&id, &name, &20, &created_at];

        let qb = QueryBuilder::builder()
            .table_name("users")
            .columns(&["id", "name", "age", "created_at"])
            .params(params.as_slice())
            .is_returning(true)
            .ty(QueryType::Insert)
            .build();
        let (stmt, params) = qb.build_sql()?;
        assert_eq!(
            stmt,
            "INSERT INTO users (id, name, age, created_at) VALUES ($1, $2, $3, $4) RETURNING id, name, age, created_at"
        );
        assert_eq!(params.len(), 4);

        Ok(())
    }

    #[tokio::test]
    async fn test_build_insert_many_sql() -> anyhow::Result<()> {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        let name = "test".to_string();
        let age = 20;
        let created_at = NaiveDateTime::parse_from_str("2024-01-07 12:34:56", "%Y-%m-%d %H:%M:%S")?;
        let params: Vec<&(dyn ToSql + Sync)> = vec![
            &id,
            &name,
            &age,
            &created_at,
            &id,
            &name,
            &age,
            &created_at,
            &id,
            &name,
            &age,
            &created_at,
        ];

        let qb = QueryBuilder::builder()
            .table_name("users")
            .columns(&["id", "name", "age", "created_at"])
            .params(params.as_slice())
            .is_returning(true)
            .ty(QueryType::Insert)
            .build();
        let (stmt, params) = qb.build_sql()?;
        assert_eq!(
            stmt,
            "INSERT INTO users (id, name, age, created_at) VALUES ($1, $2, $3, $4), ($5, $6, $7, $8), ($9, $10, $11, $12) RETURNING id, name, age, created_at"
        );
        assert_eq!(params.len(), 12);

        Ok(())
    }

    #[tokio::test]
    async fn test_build_delete_sql() -> anyhow::Result<()> {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        let columns = &["id", "name", "age", "created_at"];
        let predicates = &[equals("id", &id)];

        let qb = QueryBuilder::builder()
            .table_name("users")
            .columns(columns)
            .predicates(predicates)
            .ty(QueryType::Delete)
            .is_returning(true)
            .build();
        let (stmt, params) = qb.build_sql()?;
        assert_eq!(
            stmt,
            "DELETE FROM users WHERE id = $1 RETURNING id, name, age, created_at"
        );
        assert_eq!(params.len(), 1);

        Ok(())
    }
}
