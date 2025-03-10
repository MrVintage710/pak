#![doc = include_str!("../docs/queries.md")]

use std::{collections::HashSet, ops::{BitAnd, BitOr}};
use crate::{error::PakResult, pointer::PakTypedPointer};
use super::{value::PakValue, Pak};

//==============================================================================================
//        Pak Query
//==============================================================================================

pub trait PakQueryExpression {
    fn execute(&self, pak : &Pak) -> PakResult<HashSet<PakTypedPointer>>;
}

pub struct PakQueryUnion(Box<dyn PakQueryExpression>, Box<dyn PakQueryExpression>);

impl PakQueryExpression for PakQueryUnion {
    fn execute(&self, pak : &Pak) -> PakResult<HashSet<PakTypedPointer>> {
        let results_a = self.0.execute(pak)?;
        let results_b = self.1.execute(pak)?;
        let results = results_a.into_iter().chain(results_b.into_iter()).collect::<HashSet<_>>();
        Ok(results)
    }
}

impl<B> BitOr<B> for PakQueryUnion where B : PakQueryExpression + 'static {
    type Output = Self;

    fn bitor(self, other: B) -> Self::Output {
        PakQueryUnion(Box::new(self), Box::new(other))
    }
}

impl <B> BitOr<B> for PakQueryIntersection where B : PakQueryExpression + 'static {
    type Output = PakQueryUnion;

    fn bitor(self, other: B) -> Self::Output {
        PakQueryUnion(Box::new(self), Box::new(other))
    }
}

impl <B> BitOr<B> for PakQuery where B : PakQueryExpression + 'static {
    type Output = PakQueryUnion;

    fn bitor(self, other: B) -> Self::Output {
        PakQueryUnion(Box::new(self), Box::new(other))
    }
}

//==============================================================================================
//        Pak Query Intersection
//==============================================================================================

pub struct PakQueryIntersection(Box::<dyn PakQueryExpression>, Box::<dyn PakQueryExpression>);

impl PakQueryExpression for PakQueryIntersection {
    fn execute(&self, pak : &Pak) -> PakResult<HashSet<PakTypedPointer>> {
        let results_a = self.0.execute(pak)?;
        let results_b = self.1.execute(pak)?;
        Ok(results_a.into_iter().filter(|e| results_b.contains(e)).collect())
    }
}

impl <B> BitAnd<B> for PakQuery where B : PakQueryExpression + 'static {
    type Output = PakQueryIntersection;

    fn bitand(self, rhs: B) -> Self::Output {
        PakQueryIntersection(Box::new(self), Box::new(rhs))
    }
}

impl <B> BitAnd<B> for PakQueryUnion where B : PakQueryExpression + 'static {
    type Output = PakQueryIntersection;

    fn bitand(self, rhs: B) -> Self::Output {
        PakQueryIntersection(Box::new(self), Box::new(rhs))
    }
}

impl <B> BitAnd<B> for PakQueryIntersection where B : PakQueryExpression + 'static {
    type Output = PakQueryIntersection;

    fn bitand(self, rhs: B) -> Self::Output {
        PakQueryIntersection(Box::new(self), Box::new(rhs))
    }
}


//==============================================================================================
//        Pak Query Expression
//==============================================================================================

pub enum PakQuery {
    Equal(String, PakValue),
    GreaterThan(String, PakValue),
    LessThan(String, PakValue),
    GreaterThanEqual(String, PakValue),
    LessThanEqual(String, PakValue),
}

impl PakQuery {
    pub fn equals(key : &str, value : impl Into<PakValue>) -> Self {
        PakQuery::Equal(key.to_string(), value.into())
    }

    pub fn greater_than(key : &str, value : impl Into<PakValue>) -> Self {
        PakQuery::GreaterThan(key.to_string(), value.into())
    }

    pub fn less_than(key : &str, value : impl Into<PakValue>) -> Self {
        PakQuery::LessThan(key.to_string(), value.into())
    }
    
    pub fn greater_than_or_equal(key : &str, value : impl Into<PakValue>) -> Self {
        PakQuery::GreaterThanEqual(key.to_string(), value.into())
    }
    
    pub fn less_than_or_equal(key : &str, value : impl Into<PakValue>) -> Self {
        PakQuery::LessThanEqual(key.to_string(), value.into())
    }
}

pub fn equals(key : &str, value : impl Into<PakValue>) -> PakQuery {
    PakQuery::Equal(key.to_string(), value.into())
}

pub fn greater_than(key : &str, value : impl Into<PakValue>) -> PakQuery {
    PakQuery::GreaterThan(key.to_string(), value.into())
}

pub fn less_than(key : &str, value : impl Into<PakValue>) -> PakQuery {
    PakQuery::LessThan(key.to_string(), value.into())
}

pub fn greater_than_equal(key : &str, value : impl Into<PakValue>) -> PakQuery {
    PakQuery::GreaterThanEqual(key.to_string(), value.into())
}

pub fn less_than_equal(key : &str, value : impl Into<PakValue>) -> PakQuery {
    PakQuery::LessThanEqual(key.to_string(), value.into())
}

impl PakQueryExpression for PakQuery {
    fn execute(&self, pak : &Pak) -> PakResult<HashSet<PakTypedPointer>> {
        match self {
            PakQuery::Equal(key, pak_value) => {
                let tree = pak.get_tree(key)?;
                tree.get(pak_value)
            },
            PakQuery::GreaterThan(key, pak_value) => {
                let tree = pak.get_tree(key)?;
                tree.get_greater(pak_value)
            },
            PakQuery::LessThan(key, pak_value) => {
                let tree = pak.get_tree(key)?;
                tree.get_less(pak_value)
            },
            PakQuery::GreaterThanEqual(key, pak_value) => {
                let tree = pak.get_tree(key)?;
                tree.get_greater_eq(pak_value)
            },
            PakQuery::LessThanEqual(key, pak_value) => {
                let tree = pak.get_tree(key)?;
                tree.get_less_eq(pak_value)
            },
        }
    }
}