use bytes::{BufMut, BytesMut};
#[cfg(test)]
use googletest::description::Description;
#[cfg(test)]
use googletest::matcher::MatcherResult;
#[cfg(test)]
use googletest::prelude::Matcher;
use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::error::Error;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Vector(pub(crate) Vec<f32>);

impl From<Vec<f32>> for Vector {
    fn from(v: Vec<f32>) -> Self {
        Vector(v)
    }
}

impl From<Vector> for Vec<f32> {
    fn from(val: Vector) -> Self {
        val.0
    }
}

impl Vector {
    /// Returns a copy of the vector as a `Vec<f32>`.
    pub fn to_vec(&self) -> Vec<f32> {
        self.0.clone()
    }

    /// Returns the vector as a slice.
    pub fn as_slice(&self) -> &[f32] {
        self.0.as_slice()
    }

    pub(crate) fn from_sql(buf: &[u8]) -> Result<Vector, Box<dyn Error + Sync + Send>> {
        let dim = u16::from_be_bytes(buf[0..2].try_into()?) as usize;
        let unused = u16::from_be_bytes(buf[2..4].try_into()?);
        if unused != 0 {
            return Err("expected unused to be 0".into());
        }

        let mut vec = Vec::with_capacity(dim);
        for i in 0..dim {
            let s = 4 + 4 * i;
            vec.push(f32::from_be_bytes(buf[s..s + 4].try_into()?));
        }

        Ok(Vector(vec))
    }
}

impl<'a> FromSql<'a> for Vector {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<Vector, Box<dyn Error + Sync + Send>> {
        Vector::from_sql(raw)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "vector"
    }
}

impl ToSql for Vector {
    fn to_sql(&self, _ty: &Type, w: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        let dim = self.0.len();
        w.put_u16(dim.try_into()?);
        w.put_u16(0);

        for v in &self.0 {
            w.put_f32(*v);
        }

        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "vector"
    }

    to_sql_checked!();
}

impl<'a> IntoIterator for &'a Vector {
    type Item = f32;
    type IntoIter = VectorIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        VectorIterator {
            vec: self,
            index: 0,
        }
    }
}

pub struct VectorIterator<'a> {
    vec: &'a Vector,
    index: usize,
}

impl<'a> Iterator for VectorIterator<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.0.len() {
            let item = self.vec.0[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into() {
        let vec = Vector::from(vec![1.0, 2.0, 3.0]);
        let f32_vec: Vec<f32> = vec.into();
        assert_eq!(f32_vec, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_to_vec() {
        let vec = Vector::from(vec![1.0, 2.0, 3.0]);
        assert_eq!(vec.to_vec(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_as_slice() {
        let vec = Vector::from(vec![1.0, 2.0, 3.0]);
        assert_eq!(vec.as_slice(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_serialize() {
        let vec = Vector::from(vec![1.0, 2.0, 3.0]);
        let json = serde_json::to_string(&vec).unwrap();
        assert_eq!(json, "[1.0,2.0,3.0]");
    }

    #[test]
    fn test_deserialize() {
        let json = "[1.0,2.0,3.0]";
        let vec: Vector = serde_json::from_str(json).unwrap();
        assert_eq!(vec, Vector::from(vec![1.0, 2.0, 3.0]));
    }
}
