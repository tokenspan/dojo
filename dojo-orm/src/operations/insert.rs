use std::fmt::Debug;

use anyhow::Result;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use postgres_types::ToSql;
use tokio_postgres::NoTls;

use crate::execution::Execution;
use crate::operations::on_conflict::OnConflictOperation;
use crate::query_builder::{QueryBuilder, QueryType};
use crate::Model;

pub struct InsertOperation<'a, T>
where
    T: Model + Debug,
{
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) data: &'a [&'a T],
}

impl<'a, T> InsertOperation<'a, T>
where
    T: Model + Debug,
{
    fn build_query(&self, params: &'a [&'a (dyn ToSql + Sync)]) -> QueryBuilder {
        QueryBuilder::builder()
            .table_name(T::NAME)
            .columns(T::COLUMNS)
            .params(&params)
            .ty(QueryType::Insert)
            .is_returning(true)
            .build()
    }

    pub fn on_conflict(&self, target: &'a [&'a str]) -> OnConflictOperation<'a, T> {
        OnConflictOperation {
            pool: self.pool,
            data: self.data,
            target,
        }
    }

    pub async fn all(&self) -> Result<Vec<T>> {
        if self.data.is_empty() {
            return Ok(Vec::new());
        }

        let mut params = Vec::new();
        for data in self.data {
            params.extend(data.params());
        }

        let qb = self.build_query(&params);
        let execution = Execution::new(&self.pool, &qb);
        execution.all().await
    }

    pub async fn first(&self) -> Result<Option<T>> {
        let params = if let Some(data) = self.data.first() {
            data.params()
        } else {
            return Ok(None);
        };

        let qb = self.build_query(&params);
        let execution = Execution::new(&self.pool, &qb);
        execution.first().await
    }
}
