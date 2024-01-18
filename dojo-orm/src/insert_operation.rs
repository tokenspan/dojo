use std::fmt::Debug;
use std::marker::PhantomData;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use postgres_types::ToSql;
use tokio_postgres::NoTls;

use crate::execution::Execution;
use crate::query_builder::{QueryBuilder, QueryType};
use crate::Model;

pub struct InsertOperation<'a, T>
where
    T: Model + Debug,
{
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) data: &'a T,
}

impl<'a, T> InsertOperation<'a, T>
where
    T: Model + Debug,
{
    pub async fn exec(&self) -> anyhow::Result<T>
    where
        T: Model + Debug,
    {
        let params = self.data.params();
        let qb = QueryBuilder::builder()
            .table_name(T::NAME)
            .columns(T::COLUMNS)
            .params(&params)
            .ty(QueryType::Insert)
            .is_returning(true)
            .build();

        let execution = Execution::new(&self.pool, &qb);
        execution.first_or_throw().await
    }
}

pub struct InsertManyOperation<'a, T> {
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) data: &'a [T],
}

impl<'a, T> InsertManyOperation<'a, T> {
    pub async fn exec(&self) -> anyhow::Result<Vec<T>>
    where
        T: Model + Debug,
    {
        if self.data.is_empty() {
            return Ok(vec![]);
        }

        let mut params = vec![];
        for data in self.data {
            params.extend(data.params());
        }

        let qb = QueryBuilder::builder()
            .table_name(T::NAME)
            .columns(T::COLUMNS)
            .params(&params)
            .ty(QueryType::Insert)
            .is_returning(true)
            .build();

        let execution = Execution::new(&self.pool, &qb);
        execution.all().await
    }
}
