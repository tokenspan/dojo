use std::fmt::Debug;
use std::marker::PhantomData;

use anyhow::Result;
use tokio_postgres::Row;

use crate::model::{Model, UpdateModel};
use crate::operations::*;
use crate::pool::*;

#[derive(Clone)]
pub struct Database {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl Database {
    pub async fn new(url: &str) -> Result<Self> {
        let manager = PostgresConnectionManager::new_from_stringlike(url, NoTls)?;
        let pool = Pool::builder().build(manager).await?;

        Ok(Self { pool })
    }

    pub async fn get(&self) -> Result<PooledConnection<PostgresConnectionManager<NoTls>>> {
        Ok(self.pool.get().await?)
    }

    pub fn bind<T>(&self) -> SelectOperation<T>
    where
        T: Model + Debug,
    {
        SelectOperation {
            pool: &self.pool,
            columns: T::COLUMNS,
            params: vec![],
            predicates: vec![],
            order_by: vec![],
            _t: PhantomData::<T>,
        }
    }

    pub fn insert<'a, T>(&'a self, data: &'a [&'a T]) -> InsertOperation<'a, T>
    where
        T: Model + Debug,
    {
        InsertOperation {
            pool: &self.pool,
            data,
        }
    }

    pub fn update<'a, T, U>(&'a self, data: &'a U) -> UpdateOperation<'a, T, U>
    where
        T: Model + Debug,
        U: UpdateModel,
    {
        UpdateOperation {
            pool: &self.pool,
            columns: data.columns(),
            params: data.params(),
            predicates: vec![],
            _t: PhantomData,
            _u: PhantomData,
        }
    }

    pub fn delete<T>(&self) -> DeleteOperation<T>
    where
        T: Model + Debug,
    {
        DeleteOperation {
            pool: &self.pool,
            predicates: vec![],
            _t: PhantomData,
        }
    }

    pub async fn raw_query(&self, query: &str) -> Result<Vec<Row>> {
        let conn = self.pool.get().await?;
        conn.query(query, &[]).await.map_err(Into::into)
    }
}
