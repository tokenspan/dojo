use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::execution;
use crate::execution::Execution;
use tracing::debug;

use crate::model::Model;
use crate::order_by::OrderPredicate;
use crate::pagination::{Cursor, DefaultSortKeys, Pagination};
use crate::pool::*;
use crate::predicates::{Expr, ExprValueType, WherePredicate};
use crate::query_builder::{QueryBuilder, QueryType};
use crate::types::ToSql;
use anyhow::Result;

pub struct SelectOperation<'a, T>
where
    T: Model + Debug,
{
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) params: Vec<&'a (dyn ToSql + Sync)>,
    pub(crate) columns: &'a [&'a str],
    pub(crate) order_by: Vec<OrderPredicate<'a>>,
    pub(crate) predicates: Vec<WherePredicate<'a>>,
    pub(crate) _t: PhantomData<T>,
}

impl<'a, T> SelectOperation<'a, T>
where
    T: Model + Debug,
{
    pub fn where_by(&'a mut self, predicate: WherePredicate<'a>) -> &'a mut Self {
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
    ) -> Result<Pagination<T>> {
        let qb = QueryBuilder::builder()
            .table_name(T::NAME)
            .default_keys(T::sort_keys())
            .columns(self.columns)
            .params(&self.params)
            .where_predicates(&self.predicates)
            .first(first)
            .after(&after)
            .last(last)
            .before(&before)
            .ty(QueryType::Paging)
            .build();

        let execution = Execution::new(self.pool, &qb);
        let query_all_fut = execution.all::<T>();
        let query_count_fut = self.count();

        let (records, count) = tokio::try_join!(query_all_fut, query_count_fut)?;

        debug!(?records);
        debug!(?count);

        Ok(Pagination::new(records, first, after, last, before, count))
    }

    fn build_query_by_limit(&'a self, limit: i64) -> QueryBuilder<'a> {
        QueryBuilder::builder()
            .table_name(T::NAME)
            .columns(self.columns)
            .params(&self.params)
            .where_predicates(&self.predicates)
            .order_by_predicates(&self.order_by)
            .ty(QueryType::Select)
            .limit(limit)
            .build()
    }

    pub async fn count(&'a self) -> Result<i64> {
        let qb = QueryBuilder::builder()
            .table_name(T::NAME)
            .columns(&["COUNT(*) as count"])
            .params(&self.params)
            .where_predicates(&self.predicates)
            .ty(QueryType::Select)
            .build();

        let execution = Execution::new(self.pool, &qb);
        let row = execution.query_one().await?;

        let count = row.get("count");

        Ok(count)
    }

    pub async fn limit(&'a self, limit: i64) -> Result<Vec<T>> {
        let qb = self.build_query_by_limit(limit);

        let execution = Execution::new(self.pool, &qb);
        execution.all().await
    }

    pub async fn first(&'a self) -> Result<Option<T>> {
        let qb = self.build_query_by_limit(1);

        let execution = Execution::new(self.pool, &qb);
        execution.first().await
    }

    pub async fn all(&'a self) -> Result<Vec<T>> {
        let qb = self.build_query_by_limit(500);

        let execution = Execution::new(self.pool, &qb);
        execution.all().await
    }
}
