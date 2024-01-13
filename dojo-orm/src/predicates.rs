use std::borrow::Cow;

use crate::types::ToSql;

#[derive(Debug, Copy, Clone)]
pub enum ExprValueType {
    Value,
    Array,
}

#[derive(Debug, Clone)]
pub struct Expr<'a> {
    pub ty: ExprValueType,
    pub column: Cow<'a, str>,
    pub condition: &'a str,
    pub value: &'a (dyn ToSql + Sync),
}

#[derive(Debug, Clone)]
pub enum Predicate<'a> {
    Value(Expr<'a>),
    And(&'a [Predicate<'a>]),
    Or(&'a [Predicate<'a>]),
    Empty
}

impl<'a> Default for Predicate<'a> {
    fn default() -> Self {
        Predicate::Empty
    }
}

impl<'a> Predicate<'a> {
    pub fn to_sql(
        &self,
        params_index: &mut usize,
    ) -> (Option<String>, Vec<&'a (dyn ToSql + Sync)>) {
        match self {
            Predicate::Value(expr) => {
                let query = match expr.ty {
                    ExprValueType::Value => {
                        format!("{} {} ${}", expr.column, expr.condition, params_index)
                    }
                    ExprValueType::Array => {
                        format!("{} {} ANY(${})", expr.column, expr.condition, params_index)
                    }
                };
                let params = vec![expr.value];
                *params_index += 1;
                (Some(query), params)
            }
            Predicate::And(predicates) => {
                if predicates.is_empty() {
                    return (None, vec![]);
                }

                let mut results = vec![];
                let mut params = vec![];
                for predicate in *predicates {
                    let (q, p) = predicate.to_sql(params_index);
                    if let Some(q) = q {
                        results.push(q);
                        params.extend_from_slice(&p);
                    }
                }

                let query = format!("({})", results.join(" AND "));
                (Some(query), params)
            }
            Predicate::Or(predicates) => {
                if predicates.is_empty() {
                    return (None, vec![]);
                }

                let mut results = vec![];
                let mut params = vec![];
                for predicate in *predicates {
                    let (q, p) = predicate.to_sql(params_index);
                    if let Some(q) = q {
                        results.push(q);
                        params.extend_from_slice(&p);
                    }
                }

                let query = format!("({})", results.join(" OR "));
                (Some(query), params)
            }
            Predicate::Empty => (None, vec![]),
        }
    }
}

pub fn and<'a>(predicates: &'a [Predicate<'a>]) -> Predicate<'a> {
    Predicate::And(predicates)
}

pub fn or<'a>(predicates: &'a [Predicate<'a>]) -> Predicate<'a> {
    Predicate::Or(predicates)
}

pub fn equals<'a, T: ToSql + Sync>(column: &'a str, value: &'a T) -> Predicate<'a> {
    Predicate::Value(Expr {
        ty: ExprValueType::Value,
        column: column.into(),
        condition: "=",
        value,
    })
}

pub fn in_list<'a, T: ToSql + Sync>(column: &'a str, values: &'a T) -> Predicate<'a> {
    Predicate::Value(Expr {
        ty: ExprValueType::Array,
        column: column.into(),
        condition: "=",
        value: values,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_and() {
        // use super::{and, eq, in_list, Op};
        // use crate::types::ToSql;
        //
        // let op = and(&[eq("foo", &1), eq("bar", &2), in_list("baz", &vec![3, 4, 5])]);
        //
        // match op {
        //     Op::And(predicates) => {
        //         assert_eq!(predicates.len(), 3);
        //         match &predicates[0] {
        //             Op::Value(op_value) => {
        //                 assert_eq!(op_value.column, "foo");
        //                 assert_eq!(op_value.op, "=");
        //                 assert_eq!(
        //                     op_value
        //                         .value
        //                         .to_sql(&crate::types::Type::INT4, &mut vec![])
        //                         .unwrap(),
        //                     1
        //                 );
        //             }
        //             _ => panic!("Expected Op::Value"),
        //         }
        //         match &predicates[1] {
        //             Op::Value(op_value) => {
        //                 assert_eq!(op_value.column, "bar");
        //                 assert_eq!(op_value.op, "=");
        //                 assert_eq!(
        //                     op_value
        //                         .value
        //                         .to_sql(&crate::types::Type::INT4, &mut vec![])
        //                         .unwrap(),
        //                     2
        //                 );
        //             }
        //             _ => panic!("Expected Op::Value"),
        //         }
        //         match &predicates[2] {
        //             Op::Value(op_value) => {
        //                 assert_eq!(op_value.column, "baz");
        //                 assert_eq!(op_value.op, "IN");
        //                 assert_eq!(
        //                     op_value
        //                         .value
        //                         .to_sql(&crate::types::Type::INT4, &mut vec![])
        //                         .unwrap(),
        //                     vec![3, 4, 5]
        //                 );
        //             }
        //             _ => panic!("Expected Op::Value"),
        //         }
        //     }
        //     _ => panic!("Expected Op::And"),
        // }
    }
}
