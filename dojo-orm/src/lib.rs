#![allow(dead_code)]
#![allow(unused_imports)]

pub use database::*;
pub use model::*;

mod database;
mod execution;
mod insert_operation;
mod model;
pub mod order_by;
pub mod pagination;
pub mod predicates;
mod query_builder;
mod where_delete;
mod where_select;
mod where_update;

pub mod prelude {
    pub use crate::order_by::*;
    pub use crate::predicates::*;
}

pub mod types {
    pub use postgres_types::*;
}

pub mod pool {
    pub use bb8::Pool;
    pub use bb8::PooledConnection;
    pub use bb8_postgres::PostgresConnectionManager;
    pub use tokio_postgres::NoTls;
}

pub mod bytes {
    pub use bytes::*;
}
