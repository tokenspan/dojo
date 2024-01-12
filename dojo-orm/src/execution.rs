use std::fmt::Debug;
use std::marker::PhantomData;

use anyhow::Result;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use postgres_types::FromSql;
use tokio_postgres::{NoTls, Row};
use tracing::{debug, info};

use crate::query_builder::QueryBuilder;
use crate::Model;

pub struct Execution<'a> {
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) qb: &'a QueryBuilder<'a>,
}

impl<'a> Execution<'a> {
    pub fn new(pool: &'a Pool<PostgresConnectionManager<NoTls>>, qb: &'a QueryBuilder<'a>) -> Self {
        Self { pool, qb }
    }

    pub async fn first_or_throw<T: Model + Debug>(&self) -> Result<T> {
        let conn = self.pool.get().await?;
        let (stmt, params) = self.qb.build_sql()?;
        info!(stmt);

        let row = conn.query_one(&stmt, &params).await?;
        let record = T::from_row(row)?;
        info!(?record);

        Ok(record)
    }

    pub async fn query_one(&self) -> Result<Row> {
        let conn = self.pool.get().await?;
        let (stmt, params) = self.qb.build_sql()?;
        info!(stmt);

        let record = conn.query_one(&stmt, &params).await?;
        info!(?record);

        Ok(record)
    }

    pub async fn first<T: Model + Debug>(&self) -> Result<Option<T>> {
        let conn = self.pool.get().await?;
        let (stmt, params) = self.qb.build_sql()?;
        info!(stmt);

        let record = conn.query_opt(&stmt, &params).await?.map(T::from_row);
        let record = record.transpose()?;
        info!(?record);

        Ok(record)
    }

    pub async fn all<T: Model + Debug>(&self) -> Result<Vec<T>> {
        let conn = self.pool.get().await?;
        let (stmt, params) = self.qb.build_sql()?;
        info!(stmt);

        let rows = conn.query(&stmt, &params).await?;
        let mut records = vec![];
        for row in rows {
            records.push(T::from_row(row)?);
        }
        info!(?records);

        Ok(records)
    }
}
