use std::fmt::Debug;

use anyhow::Result;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use postgres_types::ToSql;
use tokio_postgres::NoTls;

use crate::execution::Execution;
use crate::query_builder::{QueryBuilder, QueryType};
use crate::Model;

pub struct DoOperation<'a, T>
where
    T: Model + Debug,
{
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) data: &'a [&'a T],
    pub(crate) target: &'a [&'a str],
    pub(crate) updates: &'a [(&'a str, &'a (dyn ToSql + Sync))],
}

impl<'a, T> DoOperation<'a, T>
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
            .conflict_target(self.target)
            .conflict_update(self.updates)
            .on_conflict(true)
            .build()
    }

    pub async fn all(&self) -> Result<Vec<T>> {
        let mut params = vec![];
        for p in self.data {
            params.extend(p.params());
        }

        let qb = self.build_query(&params);
        let execution = Execution::new(&self.pool, &qb);
        execution.all().await
    }

    pub async fn first(&self) -> Result<Option<T>> {
        let mut params = vec![];
        for p in self.data {
            params.extend(p.params());
        }

        let qb = self.build_query(&params);
        let execution = Execution::new(&self.pool, &qb);
        execution.first().await
    }

    pub async fn first_or_throw(&self) -> Result<T> {
        let params = if let Some(data) = self.data.first() {
            data.params()
        } else {
            return Err(anyhow::anyhow!("no data to insert"));
        };

        let qb = self.build_query(&params);
        let execution = Execution::new(&self.pool, &qb);
        execution.first_or_throw().await
    }
}
