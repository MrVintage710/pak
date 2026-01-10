use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::{group::DeserializeGroup, pointer::PakUntypedPointer, query::PakQuery, value::IntoPakValue};

use super::value::PakValue;

pub type PakIndices = HashMap<PakValue, Vec<PakUntypedPointer>>;

//==============================================================================================
//        PakItem Traits
//==============================================================================================

pub trait PakSearchable {
    fn get_indices(&self, indices : &mut Indices);
}

#[derive(Default)]
pub struct Indices(Vec<PakIndex>);

impl Indices {
    pub fn add<I, V>(&mut self, key : I, value : V) where I : PakIndexIdentifier, V : IntoPakValue {
        self.0.push(PakIndex::new(key, value));
    }
    
    pub(crate) fn unwrap(self) -> Vec<PakIndex> {
        self.0
    }
}

//==============================================================================================
//        PakIndex
//==============================================================================================

#[derive(PartialEq, Debug, Clone, PartialOrd, Deserialize, Serialize)]
pub(crate) struct PakIndex {
    pub key : String,
    pub value : PakValue
}

impl PakIndex {
    pub fn new<I, V>(key : I, value : V) -> Self where I : PakIndexIdentifier, V : IntoPakValue {
        Self {
            key: key.identifier().to_string(),
            value: value.into_pak_value(),
        }
    }
}

//==============================================================================================
//        PakIndexIdentifier
//==============================================================================================

pub trait PakIndexIdentifier {
    fn identifier(&self) -> &str;
    
    fn equals<T, V>(&self, other: V) -> PakQuery<T> where T : DeserializeGroup, V : IntoPakValue {
        PakQuery::equals(self.identifier(), other.into_pak_value())
    }
    
    fn less_than<T, V>(&self, other: V) -> PakQuery<T> where T : DeserializeGroup, V : IntoPakValue {
        PakQuery::less_than(self.identifier(), other.into_pak_value())
    }
    
    fn greater_than<T, V>(&self, other: V) -> PakQuery<T> where T : DeserializeGroup, V : IntoPakValue {
        PakQuery::greater_than(self.identifier(), other.into_pak_value())
    }
    
    fn greater_than_or_equal<T, V>(&self, other: V) -> PakQuery<T> where T : DeserializeGroup, V : IntoPakValue {
        PakQuery::greater_than_or_equal(self.identifier(), other.into_pak_value())
    }
    
    fn less_than_or_equal<T, V>(&self, other: V) -> PakQuery<T> where T : DeserializeGroup, V : IntoPakValue {
        PakQuery::less_than_or_equal(self.identifier(), other.into_pak_value())
    }
    
    fn contains_value<T, V>(&self, other : V) -> PakQuery<T> where T : DeserializeGroup, V : IntoPakValue {
        PakQuery::contains(self.identifier(), other.into_pak_value())
    }
}

impl PakIndexIdentifier for String {
    fn identifier(&self) -> &str {
        self
    }
}

impl <'id> PakIndexIdentifier for &'id str {
    fn identifier(&self) -> &str {
        self
    }
}