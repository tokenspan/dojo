use async_graphql::Enum;
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
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Asc => write!(f, "ASC"),
            Direction::Desc => write!(f, "DESC"),
        }
    }
}
#[derive(Debug, Clone)]
pub struct OrderPredicate<'a> {
    pub column: &'a str,
    pub direction: Direction,
}

impl<'a> From<OrderPredicate<'a>> for String {
    fn from(value: OrderPredicate<'a>) -> Self {
        format!("{} {}", value.column, value.direction)
    }
}

#[derive(Debug, Default, Clone)]
pub struct OrderBy<'a>(&'a [&'a OrderPredicate<'a>]);

impl<'a> OrderBy<'a> {
    pub fn new(values: &'a [&'a OrderPredicate]) -> Self {
        Self(values)
    }

    pub fn to_sql(&self) -> String {
        let mut stmt = "".to_string();
        if let Some(value) = self.0.first() {
            stmt.push_str(value.column);
            stmt.push(' ');
            stmt.push_str(value.direction.to_string().as_str());
        }

        for value in self.0.iter().skip(1) {
            stmt.push_str(", ");
            stmt.push_str(value.column);
            stmt.push(' ');
            stmt.push_str(Direction::Asc.to_string().as_str());
        }

        stmt
    }
}

pub fn asc(column: &str) -> OrderPredicate {
    OrderPredicate {
        column,
        direction: Direction::Asc,
    }
}

pub fn desc(column: &str) -> OrderPredicate {
    OrderPredicate {
        column,
        direction: Direction::Desc,
    }
}
