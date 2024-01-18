use std::fmt::Debug;
use std::marker::PhantomData;

use crate::insert_operation::{InsertManyOperation, InsertOperation};
use crate::model::{Model, UpdateModel};
use crate::pool::*;
use crate::where_delete::WhereDelete;
use crate::where_select::WhereSelect;
use crate::where_update::WhereUpdate;

#[derive(Clone)]
pub struct Database {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl Database {
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let manager = PostgresConnectionManager::new_from_stringlike(url, NoTls)?;
        let pool = Pool::builder().build(manager).await?;

        Ok(Self { pool })
    }

    pub async fn get(&self) -> anyhow::Result<PooledConnection<PostgresConnectionManager<NoTls>>> {
        Ok(self.pool.get().await?)
    }

    pub fn bind<T>(&self) -> WhereSelect<T>
    where
        T: Model + Debug,
    {
        WhereSelect {
            pool: &self.pool,
            columns: T::COLUMNS,
            params: vec![],
            predicates: vec![],
            order_by: vec![],
            _t: PhantomData::<T>,
        }
    }

    pub async fn drop_table<T>(&self) -> anyhow::Result<()>
    where
        T: Model,
    {
        let query = format!("DROP TABLE IF EXISTS {}", T::NAME);

        let conn = self.pool.get().await?;
        conn.execute(&query, &[]).await?;

        Ok(())
    }

    pub fn insert<'a, T>(&'a self, data: &'a T) -> InsertOperation<'a, T>
    where
        T: Model + Debug,
    {
        InsertOperation {
            pool: &self.pool,
            data: &data,
        }
    }

    pub fn insert_many<'a, T>(&'a self, data: &'a [T]) -> InsertManyOperation<'a, T>
    where
        T: Model + Debug,
    {
        InsertManyOperation {
            pool: &self.pool,
            data: &data,
        }
    }

    pub fn update<'a, T, U>(&'a self, data: &'a U) -> WhereUpdate<'a, T, U>
    where
        T: Model + Debug,
        U: UpdateModel,
    {
        WhereUpdate {
            pool: &self.pool,
            columns: data.columns(),
            params: data.params(),
            predicates: vec![],
            _t: PhantomData,
            _u: PhantomData,
        }
    }

    pub fn delete<T>(&self) -> WhereDelete<T>
    where
        T: Model + Debug,
    {
        WhereDelete {
            pool: &self.pool,
            predicates: vec![],
            _t: PhantomData,
        }
    }
}
