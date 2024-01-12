use crate::execution::Execution;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::DerefMut;
use tracing::debug;

use crate::pool::*;
use crate::types::ToSql;

use crate::model::{Model, UpdateModel};
use crate::predicates::Predicate;
use crate::query_builder::{QueryBuilder, QueryType};

pub struct WhereDelete<'a, T> {
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) predicates: Vec<Predicate<'a>>,
    pub(crate) _t: PhantomData<T>,
}

impl<'a, T> WhereDelete<'a, T>
where
    T: Model + Debug,
{
    pub fn where_by(&'a mut self, predicate: Predicate<'a>) -> &'a mut Self {
        self.predicates.push(predicate);
        self
    }

    pub async fn exec(&'a self) -> anyhow::Result<T> {
        let qb = QueryBuilder::builder()
            .table_name(T::NAME)
            .columns(T::COLUMNS)
            .predicates(self.predicates.as_slice())
            .ty(QueryType::Delete)
            .is_returning(true)
            .build();

        let execution = Execution::new(self.pool, &qb);
        execution.first_or_throw().await
    }
}
