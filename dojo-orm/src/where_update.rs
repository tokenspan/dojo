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

pub struct WhereUpdate<'a, T, U> {
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) columns: Vec<&'a str>,
    pub(crate) params: Vec<&'a (dyn ToSql + Sync)>,
    pub(crate) predicates: Vec<Predicate<'a>>,
    pub(crate) _t: PhantomData<T>,
    pub(crate) _u: PhantomData<U>,
}

impl<'a, T, U> WhereUpdate<'a, T, U>
where
    T: Model + Debug,
    U: UpdateModel,
{
    pub fn where_by(&'a mut self, predicate: Predicate<'a>) -> &'a mut Self {
        self.predicates.push(predicate);
        self
    }

    pub async fn exec(&'a self) -> anyhow::Result<T> {
        let qb = QueryBuilder::builder()
            .table_name(T::NAME)
            .columns(&self.columns)
            .params(&self.params)
            .predicates(&self.predicates)
            .ty(QueryType::Update)
            .is_returning(true)
            .returning(T::COLUMNS)
            .build();

        let execution = Execution::new(self.pool, &qb);
        execution.first_or_throw().await
    }
}
