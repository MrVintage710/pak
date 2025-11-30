#![doc = include_str!("../../docs/queries.md")]

pub mod pql;

use std::{ops::{BitAnd, BitOr}, rc::Rc, sync::Arc};
use ordermap::OrderSet;

use crate::{error::PakResult, pointer::PakPointer};
use super::{value::PakValue, Pak};

//==============================================================================================
//        Pak Query
//==============================================================================================

pub trait PakQueryExpression {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>>;
}

impl PakQueryExpression for Box<dyn PakQueryExpression> {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        self.as_ref().execute(pak)
    }
}

impl PakQueryExpression for Rc<dyn PakQueryExpression> {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        Rc::as_ref(&self).execute(pak)
    }
}

impl PakQueryExpression for Arc<dyn PakQueryExpression> {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        Arc::as_ref(&self).execute(pak)
    }
}

impl <T> PakQueryExpression for Box<T> where T : PakQueryExpression {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        self.as_ref().execute(pak)
    }
}

impl <T> PakQueryExpression for Rc<T> where T : PakQueryExpression  {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        Rc::as_ref(&self).execute(pak)
    }
}

impl <T> PakQueryExpression for Arc<T> where T : PakQueryExpression {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        Arc::as_ref(&self).execute(pak)
    }
}

//==============================================================================================
//        PakQueryUnion
//==============================================================================================

pub struct PakQueryUnion(Box<dyn PakQueryExpression>, Box<dyn PakQueryExpression>);

impl PakQueryExpression for PakQueryUnion {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        let results_a = self.0.execute(pak)?;
        let results_b = self.1.execute(pak)?;
        let results = results_a.into_iter().chain(results_b.into_iter()).collect::<OrderSet<_>>();
        Ok(results)
    }
}

impl PakQueryUnion {
    pub fn new(first : impl PakQueryExpression + 'static, second : impl PakQueryExpression + 'static) -> Self {
        PakQueryUnion(Box::new(first), Box::new(second))
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

impl PakQueryIntersection {
    pub fn new(first : impl PakQueryExpression + 'static, second : impl PakQueryExpression + 'static) -> Self {
        PakQueryIntersection(Box::new(first), Box::new(second))
    } 
}

impl PakQueryExpression for PakQueryIntersection {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
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
    Contains(String, PakValue)
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
    
    pub fn contains(key : &str, value : impl Into<PakValue>) -> Self {
        PakQuery::Contains(key.to_string(), value.into())
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
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
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
            PakQuery::Contains(key, pak_value) => {
                let tree = pak.get_tree(key)?;
                tree.get_contains(pak_value)
            }
        }
    }
}