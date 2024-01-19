use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::execution;
use crate::execution::Execution;
use tracing::debug;

use crate::model::Model;
use crate::order_by::{OrderBy, OrderPredicate};
use crate::pagination::{Cursor, DefaultSortKeys, Pagination};
use crate::pool::*;
use crate::predicates::{Expr, ExprValueType, Predicate};
use crate::query_builder::{QueryBuilder, QueryType};
use crate::types::ToSql;

pub struct WhereSelect<'a, T> {
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) params: Vec<&'a (dyn ToSql + Sync)>,
    pub(crate) columns: &'a [&'a str],
    pub(crate) order_by: Vec<OrderPredicate<'a>>,
    pub(crate) predicates: Vec<Predicate<'a>>,
    pub(crate) _t: PhantomData<T>,
}

impl<'a, T> WhereSelect<'a, T>
where
    T: Model + Debug,
{
    pub fn where_by(&'a mut self, predicate: Predicate<'a>) -> &'a mut Self {
        self.predicates.push(predicate);
        self
    }

    pub fn order_by(&'a mut self, order_by: OrderPredicate<'a>) -> &'a mut Self {
        self.order_by.push(order_by);
        self
    }

    pub async fn cursor(
        &'a self,
        first: Option<i64>,
        after: Option<Cursor>,
        last: Option<i64>,
        before: Option<Cursor>,
    ) -> anyhow::Result<Pagination<T>> {
        let qb = QueryBuilder::builder()
            .table_name(T::NAME)
            .default_keys(T::sort_keys())
            .columns(self.columns)
            .params(&self.params)
            .predicates(&self.predicates)
            .first(first)
            .after(&after)
            .last(last)
            .before(&before)
            .ty(QueryType::Paging)
            .build();

        let execution = Execution::new(self.pool, &qb);
        let query_all_fut = execution.all::<T>();

        let qb = QueryBuilder::builder()
            .table_name(T::NAME)
            .columns(&["COUNT(*) as count"])
            .params(&self.params)
            .predicates(&self.predicates)
            .ty(QueryType::Paging)
            .build();

        let execution = Execution::new(self.pool, &qb);
        let query_count_fut = execution.query_one();

        let (records, row) = tokio::try_join!(query_all_fut, query_count_fut)?;
        let count = row.get("count");

        debug!(?records);
        debug!(?count);

        Ok(Pagination::new(records, first, after, last, before, count))
    }

    fn query_by_limit(&'a self, limit: i64) -> QueryBuilder<'a> {
        QueryBuilder::builder()
            .table_name(T::NAME)
            .columns(self.columns)
            .params(&self.params)
            .predicates(&self.predicates)
            .order_by(&self.order_by)
            .ty(QueryType::Select)
            .limit(limit)
            .build()
    }

    pub async fn count(&'a self) -> anyhow::Result<i64> {
        let qb = QueryBuilder::builder()
            .table_name(T::NAME)
            .columns(&["COUNT(*) as count"])
            .params(&self.params)
            .predicates(&self.predicates)
            .ty(QueryType::Select)
            .build();

        let execution = Execution::new(self.pool, &qb);
        let row = execution.query_one().await?;

        let count = row.get("count");

        Ok(count)
    }

    pub async fn limit(&'a self, limit: i64) -> anyhow::Result<Vec<T>> {
        let qb = self.query_by_limit(limit);

        let execution = Execution::new(self.pool, &qb);
        execution.all().await
    }

    pub async fn first(&'a self) -> anyhow::Result<Option<T>> {
        let qb = self.query_by_limit(1);

        let execution = Execution::new(self.pool, &qb);
        execution.first().await
    }

    pub async fn all(&'a self) -> anyhow::Result<Vec<T>> {
        let qb = self.query_by_limit(500);

        let execution = Execution::new(self.pool, &qb);
        execution.all().await
    }
}
