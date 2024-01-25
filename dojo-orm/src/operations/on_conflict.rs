use crate::Model;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use postgres_types::ToSql;
use std::fmt::Debug;
use tokio_postgres::NoTls;

use crate::operations::r#do::DoOperation;

pub struct OnConflictOperation<'a, T>
where
    T: Model + Debug,
{
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) data: &'a [&'a T],
    pub(crate) target: &'a [&'a str],
}

impl<'a, T> OnConflictOperation<'a, T>
where
    T: Model + Debug,
{
    pub fn do_nothing(&self) -> DoOperation<'a, T> {
        DoOperation {
            pool: self.pool,
            data: self.data,
            target: self.target,
            updates: &[],
        }
    }

    pub fn do_update(
        &self,
        updates: &'a [(&'a str, &'a (dyn ToSql + Sync))],
    ) -> DoOperation<'a, T> {
        DoOperation {
            pool: self.pool,
            data: self.data,
            target: self.target,
            updates,
        }
    }
}
