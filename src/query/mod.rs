#![doc = include_str!("../../docs/queries.md")]

pub mod pql;

use std::{marker::PhantomData, ops::{BitAnd, BitOr}, rc::Rc, sync::Arc};
use ordermap::OrderSet;

use crate::{error::PakResult, group::DeserializeGroup, pointer::PakPointer};
use super::{value::PakValue, Pak};

//==============================================================================================
//        Pak Query
//==============================================================================================

pub trait PakQueryExpression<T> where T : DeserializeGroup {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>>;
}

impl <T, Q> PakQueryExpression<T> for Vec<Q> where T : DeserializeGroup, Q : PakQueryExpression<T> {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        let results = self.iter()
            .filter_map(|expression| expression.execute(pak).ok())
            .flatten()
            .collect::<OrderSet<_>>()
        ;
        Ok(results)
    }
}

impl <T> PakQueryExpression<T> for Box<dyn PakQueryExpression<T>> where T : DeserializeGroup {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        self.as_ref().execute(pak)
    }
}

impl <T> PakQueryExpression<T> for Rc<dyn PakQueryExpression<T>> where T : DeserializeGroup {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        Rc::as_ref(&self).execute(pak)
    }
}

impl <T> PakQueryExpression<T> for Arc<dyn PakQueryExpression<T>> where T : DeserializeGroup {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        Arc::as_ref(&self).execute(pak)
    }
}

impl <T, Q> PakQueryExpression<T> for Box<Q> where T : DeserializeGroup, Q : PakQueryExpression<T> {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        self.as_ref().execute(pak)
    }
}

impl <T, Q> PakQueryExpression<T> for Rc<Q> where T : DeserializeGroup, Q : PakQueryExpression<T>  {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        Rc::as_ref(&self).execute(pak)
    }
}

impl <T, Q> PakQueryExpression<T> for Arc<Q> where T : DeserializeGroup, Q : PakQueryExpression<T> {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        Arc::as_ref(&self).execute(pak)
    }
}

//==============================================================================================
//        PakQueryUnion
//==============================================================================================

pub struct PakQueryUnion<T>(Box<dyn PakQueryExpression<T>>, Box<dyn PakQueryExpression<T>>) where T : DeserializeGroup;

impl <T> PakQueryExpression<T> for PakQueryUnion<T> where T : DeserializeGroup {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        let results_a = self.0.execute(pak)?;
        let results_b = self.1.execute(pak)?;
        let results = results_a.into_iter().chain(results_b.into_iter()).collect::<OrderSet<_>>();
        Ok(results)
    }
}

impl <T> PakQueryUnion<T> where T : DeserializeGroup {
    pub fn new(first : impl PakQueryExpression<T> + 'static, second : impl PakQueryExpression<T> + 'static) -> Self {
        PakQueryUnion(Box::new(first), Box::new(second))
    } 
}

impl <T, B> BitOr<B> for PakQueryUnion<T> where T : DeserializeGroup + 'static, B : PakQueryExpression<T> + 'static {
    type Output = Self;

    fn bitor(self, other: B) -> Self::Output {
        PakQueryUnion(Box::new(self), Box::new(other))
    }
}

impl <T, B> BitOr<B> for PakQueryIntersection<T> where T : DeserializeGroup + 'static, B : PakQueryExpression<T> + 'static {
    type Output = PakQueryUnion<T>;

    fn bitor(self, other: B) -> Self::Output {
        PakQueryUnion(Box::new(self), Box::new(other))
    }
}

impl <T, B> BitOr<B> for PakQuery<T> where T : DeserializeGroup + 'static, B : PakQueryExpression<T> + 'static {
    type Output = PakQueryUnion<T>;

    fn bitor(self, other: B) -> Self::Output {
        PakQueryUnion(Box::new(self), Box::new(other))
    }
}

//==============================================================================================
//        Pak Query Intersection
//==============================================================================================

pub struct PakQueryIntersection<T>(Box::<dyn PakQueryExpression<T>>, Box::<dyn PakQueryExpression<T>>) where T : DeserializeGroup;

impl <T> PakQueryIntersection<T> where T : DeserializeGroup {
    pub fn new(first : impl PakQueryExpression<T> + 'static, second : impl PakQueryExpression<T> + 'static) -> Self {
        PakQueryIntersection(Box::new(first), Box::new(second))
    } 
}

impl <T> PakQueryExpression<T> for PakQueryIntersection<T> where T : DeserializeGroup {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        let results_a = self.0.execute(pak)?;
        let results_b = self.1.execute(pak)?;
        Ok(results_a.into_iter().filter(|e| results_b.contains(e)).collect())
    }
}

impl <T, B> BitAnd<B> for PakQuery<T> where  T : DeserializeGroup + 'static, B : PakQueryExpression<T> + 'static {
    type Output = PakQueryIntersection<T>;

    fn bitand(self, rhs: B) -> Self::Output {
        PakQueryIntersection(Box::new(self), Box::new(rhs))
    }
}

impl <T, B> BitAnd<B> for PakQueryUnion<T> where T : DeserializeGroup + 'static, B : PakQueryExpression<T> + 'static {
    type Output = PakQueryIntersection<T>;

    fn bitand(self, rhs: B) -> Self::Output {
        PakQueryIntersection(Box::new(self), Box::new(rhs))
    }
}

impl <T, B> BitAnd<B> for PakQueryIntersection<T> where T : DeserializeGroup + 'static, B : PakQueryExpression<T> + 'static {
    type Output = PakQueryIntersection<T>;

    fn bitand(self, rhs: B) -> Self::Output {
        PakQueryIntersection(Box::new(self), Box::new(rhs))
    }
}


//==============================================================================================
//        Pak Query Expression
//==============================================================================================

#[derive(Default)]
pub enum PakQuery<T : DeserializeGroup> {
    Equal(String, PakValue, PhantomData<T>),
    GreaterThan(String, PakValue),
    LessThan(String, PakValue),
    GreaterThanEqual(String, PakValue),
    LessThanEqual(String, PakValue),
    Contains(String, PakValue),
    #[default]
    All
}

impl <T> PakQuery<T> where T : DeserializeGroup {
    pub fn equals(key : &str, value : impl Into<PakValue>) -> Self {
        PakQuery::Equal(key.to_string(), value.into(), PhantomData)
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

pub fn equals<T : DeserializeGroup>(key : &str, value : impl Into<PakValue>) -> PakQuery<T> {
    PakQuery::Equal(key.to_string(), value.into(), PhantomData)
}

pub fn greater_than<T : DeserializeGroup>(key : &str, value : impl Into<PakValue>) -> PakQuery<T> {
    PakQuery::GreaterThan(key.to_string(), value.into())
}

pub fn less_than<T : DeserializeGroup>(key : &str, value : impl Into<PakValue>) -> PakQuery<T> {
    PakQuery::LessThan(key.to_string(), value.into())
}

pub fn greater_than_equal<T : DeserializeGroup>(key : &str, value : impl Into<PakValue>) -> PakQuery<T> {
    PakQuery::GreaterThanEqual(key.to_string(), value.into())
}

pub fn less_than_equal<T : DeserializeGroup>(key : &str, value : impl Into<PakValue>) -> PakQuery<T> {
    PakQuery::LessThanEqual(key.to_string(), value.into())
}

impl <T> PakQueryExpression<T> for PakQuery<T> where T : DeserializeGroup {
    fn execute(&self, pak : &Pak) -> PakResult<OrderSet<PakPointer>> {
        match self {
            PakQuery::Equal(key, pak_value, _) => {
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
            },
            PakQuery::All => {
                let list = pak.fetch_all_pointers_of::<T>()?;
                Ok(list.into_iter().collect::<OrderSet<_>>())
            }
        }
    }
}