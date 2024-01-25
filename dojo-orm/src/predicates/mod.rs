use std::borrow::Cow;

use crate::types::ToSql;

#[derive(Debug, Copy, Clone)]
pub enum ExprValueType {
    Value,
    Array,
    Function,
}

#[derive(Debug, Clone)]
pub struct ExprValue<'a> {
    pub column: Cow<'a, str>,
    pub condition: &'a str,
    pub value: &'a (dyn ToSql + Sync),
}

#[derive(Debug, Clone)]
pub struct ExprArray<'a> {
    pub column: Cow<'a, str>,
    pub condition: &'a str,
    pub values: &'a (dyn ToSql + Sync),
}

#[derive(Debug, Clone)]
pub struct ExprFunction<'a> {
    pub column: Cow<'a, str>,
    pub condition: &'a str,
    pub name: &'a str,
    pub args: &'a [&'a (dyn ToSql + Sync)],
}

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    Value(ExprValue<'a>),
    Array(ExprArray<'a>),
    Function(ExprFunction<'a>),
    Raw(String),
    RawStr(&'a str),
}

#[derive(Debug, Clone)]
pub enum WherePredicate<'a> {
    Value(Expr<'a>),
    And(&'a [WherePredicate<'a>]),
    Or(&'a [WherePredicate<'a>]),
    Empty,
}

impl<'a> Default for WherePredicate<'a> {
    fn default() -> Self {
        WherePredicate::Empty
    }
}

impl<'a> WherePredicate<'a> {
    pub fn to_sql(
        &self,
        params_index: &mut usize,
    ) -> (Option<String>, Vec<&'a (dyn ToSql + Sync)>) {
        match self {
            WherePredicate::Value(expr) => {
                let mut params: Vec<&'a (dyn ToSql + Sync)> = vec![];
                let query = match expr {
                    Expr::Value(expr) => {
                        let query = format!("{} {} ${}", expr.column, expr.condition, params_index);
                        params.push(expr.value);
                        *params_index += 1;

                        query
                    }
                    Expr::Array(expr) => {
                        let query =
                            format!("{} {} ANY(${})", expr.column, expr.condition, params_index);

                        params.push(expr.values);
                        *params_index += 1;

                        query
                    }
                    Expr::Function(expr) => {
                        let mut formatted_args = vec![];
                        for arg in expr.args {
                            params.push(*arg);
                            formatted_args.push(format!("${}", params_index));
                            *params_index += 1;
                        }

                        let query = format!(
                            "{} {} {}({})",
                            expr.column,
                            expr.condition,
                            expr.name,
                            formatted_args.join(", ")
                        );

                        query
                    }
                    Expr::Raw(raw) => raw.to_string(),
                    Expr::RawStr(raw) => raw.to_string(),
                };

                (Some(query), params)
            }
            WherePredicate::And(predicates) => {
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
            WherePredicate::Or(predicates) => {
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
            WherePredicate::Empty => (None, vec![]),
        }
    }
}

pub fn and<'a>(predicates: &'a [WherePredicate<'a>]) -> WherePredicate<'a> {
    WherePredicate::And(predicates)
}

pub fn or<'a>(predicates: &'a [WherePredicate<'a>]) -> WherePredicate<'a> {
    WherePredicate::Or(predicates)
}

pub fn equals<'a, T: ToSql + Sync>(column: &'a str, value: &'a T) -> WherePredicate<'a> {
    WherePredicate::Value(Expr::Value(ExprValue {
        column: column.into(),
        condition: "=",
        value,
    }))
}

pub fn in_list<'a>(column: &'a str, values: &'a (dyn ToSql + Sync)) -> WherePredicate<'a> {
    WherePredicate::Value(Expr::Array(ExprArray {
        column: column.into(),
        condition: "=",
        values,
    }))
}

pub fn text_search<'a>(column: &'a str, lang: &'a str, value: &'a str) -> WherePredicate<'a> {
    WherePredicate::Value(Expr::Raw(
        format!(
            "{} @@ websearch_to_tsquery('{}', '{}')",
            column, lang, value
        )
        .to_string(),
    ))
}

pub fn raw<'a>(raw: String) -> WherePredicate<'a> {
    WherePredicate::Value(Expr::Raw(raw))
}

pub fn raw_str(raw: &str) -> WherePredicate {
    WherePredicate::Value(Expr::RawStr(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_and() {
        let predicates = [
            equals("id", &1),
            equals("name", &"test"),
            equals("age", &20),
            equals("is_active", &true),
        ];
        let predicates = and(&predicates);

        let (query, params) = predicates.to_sql(&mut 1);
        assert_eq!(
            query.unwrap(),
            "(id = $1 AND name = $2 AND age = $3 AND is_active = $4)"
        );
        assert_eq!(params.len(), 4);
    }

    #[test]
    fn test_text_search() {
        let predicates = [text_search("name", "english", "test")];
        let predicates = and(&predicates);

        let (query, params) = predicates.to_sql(&mut 1);
        assert_eq!(
            query.unwrap(),
            "(name @@ websearch_to_tsquery('english', 'test'))"
        );
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_or() {
        let predicates = [
            equals("id", &1),
            equals("name", &"test"),
            equals("age", &20),
            equals("is_active", &true),
        ];
        let predicates = or(&predicates);

        let (query, params) = predicates.to_sql(&mut 1);
        assert_eq!(
            query.unwrap(),
            "(id = $1 OR name = $2 OR age = $3 OR is_active = $4)"
        );
        assert_eq!(params.len(), 4);
    }

    #[test]
    fn test_and_or() {
        let predicates = [
            equals("id", &1),
            equals("name", &"test"),
            equals("age", &20),
            equals("is_active", &true),
        ];
        let and_predicates = and(&predicates);

        let predicates = [
            and_predicates,
            equals("id", &1),
            equals("name", &"test"),
            equals("age", &20),
            equals("is_active", &true),
        ];
        let or_predicates = or(&predicates);

        let (query, params) = or_predicates.to_sql(&mut 1);
        assert_eq!(
            query.unwrap(),
            "((id = $1 AND name = $2 AND age = $3 AND is_active = $4) OR id = $5 OR name = $6 OR age = $7 OR is_active = $8)"
        );
        assert_eq!(params.len(), 8);
    }
}
