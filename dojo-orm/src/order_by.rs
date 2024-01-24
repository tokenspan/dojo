use async_graphql::Enum;
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum::EnumString;

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, EnumString, Serialize, Deserialize)]
pub enum Direction {
    #[graphql(name = "asc")]
    #[strum(serialize = "asc", serialize = "ASC")]
    Asc,
    #[graphql(name = "desc")]
    #[strum(serialize = "desc", serialize = "DESC")]
    Desc,
    #[graphql(name = "nearest")]
    #[strum(serialize = "nearest", serialize = "NEAREST")]
    Nearest,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Asc => write!(f, "ASC"),
            Direction::Desc => write!(f, "DESC"),
            Direction::Nearest => write!(f, "NEAREST"),
        }
    }
}
#[derive(Debug, Clone)]
pub enum OrderPredicate<'a> {
    Asc(&'a str),
    Desc(&'a str),
    Nearest(&'a str, &'a Vector),
}

pub fn asc(column: &str) -> OrderPredicate {
    OrderPredicate::Asc(column)
}

pub fn desc(column: &str) -> OrderPredicate {
    OrderPredicate::Desc(column)
}

pub fn nearest<'a>(column: &'a str, vector: &'a Vector) -> OrderPredicate<'a> {
    OrderPredicate::Nearest(column, vector)
}
