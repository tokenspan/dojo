use crate::pagination::{Cursor, Pagination};
use crate::Model;
use async_graphql::connection::{Connection, CursorType, Edge};
use async_graphql::{
    InputValueError, InputValueResult, OutputType, Scalar, ScalarType, SimpleObject, Value,
};
use std::fmt::Debug;

#[Scalar]
impl ScalarType for Cursor {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            let cursor =
                Cursor::decode(value).map_err(|e| InputValueError::custom(e.to_string()))?;
            Ok(cursor)
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        let value = self.encode();
        Value::String(value)
    }
}

impl CursorType for Cursor {
    type Error = anyhow::Error;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        Self::decode(s)
    }

    fn encode_cursor(&self) -> String {
        self.encode()
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct AdditionalFields {
    pub total_nodes: i64,
}

impl<T> From<Pagination<T>> for Connection<Cursor, T, AdditionalFields>
where
    T: OutputType + Model + Debug,
{
    fn from(value: Pagination<T>) -> Self {
        let mut connection = Connection::with_additional_fields(
            value.has_previous,
            value.has_next,
            AdditionalFields {
                total_nodes: value.total_nodes,
            },
        );

        connection.edges = value
            .items
            .into_iter()
            .map(|item| Edge::new(item.cursor(), item))
            .collect::<Vec<_>>();

        connection
    }
}
