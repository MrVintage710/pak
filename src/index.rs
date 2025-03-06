use std::{collections::HashMap};
use serde::{Deserialize, Serialize};
use crate::{pointer::PakUntypedPointer, value::IntoPakValue};

use super::value::PakValue;

pub type PakIndices = HashMap<PakValue, Vec<PakUntypedPointer>>;

//==============================================================================================
//        PakIndex
//==============================================================================================

#[derive(PartialEq, Debug, Clone, PartialOrd, Deserialize, Serialize)]
pub struct PakIndex {
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