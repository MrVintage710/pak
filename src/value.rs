

//==============================================================================================
//        Pak Values
//==============================================================================================

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::PakPointer;

#[derive(Deserialize, Serialize, Clone, Hash, Default)]
pub enum PakValue {
    String(String),
    Float(u64),
    Int(i64),
    Uint(u64),
    Boolean(bool),
    #[default]
    Void
}

impl PartialEq for PakValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PakValue::String(a), PakValue::String(b)) => a == b,
            (PakValue::Float(a), PakValue::Float(b)) => a == b,
            (PakValue::Float(a), PakValue::Int(b)) => f64::from_bits(*a) == *b as f64,
            (PakValue::Float(a), PakValue::Uint(b)) => f64::from_bits(*a) == *b as f64,
            (PakValue::Int(a), PakValue::Float(b)) => *a as f64 == f64::from_bits(*b),
            (PakValue::Int(a), PakValue::Int(b)) => a == b,
            (PakValue::Int(a), PakValue::Uint(b)) => *a == *b as i64,
            (PakValue::Uint(a), PakValue::Float(b)) => *a as f64 == f64::from_bits(*b),
            (PakValue::Uint(a), PakValue::Int(b)) => *a as i64 == *b,
            (PakValue::Uint(a), PakValue::Uint(b)) => a == b,
            (PakValue::Boolean(a), PakValue::Boolean(b)) => a == b,
            (PakValue::Void, PakValue::Void) => true,
            _ => false,
        }
    }
}

impl Debug for PakValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PakValue::String(string) => string.fmt(f),
            PakValue::Float(float) => float.fmt(f),
            PakValue::Int(int) => int.fmt(f),
            PakValue::Uint(uint) => uint.fmt(f),
            PakValue::Boolean(boolean) => boolean.fmt(f),
            PakValue::Void => f.write_str("Void"),
        }
    }
}

impl PartialOrd for PakValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (PakValue::String(a), PakValue::String(b)) => a.partial_cmp(b),
            (PakValue::Float(a), PakValue::Float(b)) => a.partial_cmp(b),
            (PakValue::Float(a), PakValue::Int(b)) => f64::from_bits(*a).partial_cmp(&(*b as f64)),
            (PakValue::Float(a), PakValue::Uint(b)) => f64::from_bits(*a).partial_cmp(&(*b as f64)),
            (PakValue::Int(a), PakValue::Float(b)) => (*a as f64).partial_cmp(&f64::from_bits(*b)),
            (PakValue::Int(a), PakValue::Int(b)) => a.partial_cmp(b),
            (PakValue::Int(a), PakValue::Uint(b)) => (*a as i64).partial_cmp(&(*b as i64)),
            (PakValue::Uint(a), PakValue::Float(b)) => (*a as f64).partial_cmp(&f64::from_bits(*b)),
            (PakValue::Uint(a), PakValue::Int(b)) => (*a as i64).partial_cmp(&(*b as i64)),
            (PakValue::Uint(a), PakValue::Uint(b)) => a.partial_cmp(b),
            (PakValue::Boolean(a), PakValue::Boolean(b)) => a.partial_cmp(b),
            (PakValue::Void, PakValue::Void) => Some(std::cmp::Ordering::Equal),
            _ => None,
        }
    }
}

impl Eq for PakValue {}

impl Ord for PakValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}


impl PakValue {
    pub fn as_string(&self) -> Option<String> {
        match self {
            PakValue::String(value) => Some(value.clone()),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            PakValue::Float(bits) => Some(f64::from_bits(*bits)),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self {
            PakValue::Float(bits) => Some(f64::from_bits(*bits) as f32),
            _ => None,
        }
    }
    
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            PakValue::Uint(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            PakValue::Uint(value) => Some(*value as u32),
            _ => None,
        }
    }

    pub fn as_u16(&self) -> Option<u16> {
        match self {
            PakValue::Uint(value) => Some(*value as u16),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> Option<u8> {
        match self {
            PakValue::Uint(value) => Some(*value as u8),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            PakValue::Int(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            PakValue::Int(value) => Some(*value as i32),
            _ => None,
        }
    }

    pub fn as_i16(&self) -> Option<i16> {
        match self {
            PakValue::Int(value) => Some(*value as i16),
            _ => None,
        }
    }

    pub fn as_i8(&self) -> Option<i8> {
        match self {
            PakValue::Int(value) => Some(*value as i8),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PakValue::Boolean(value) => Some(*value),
            _ => None,
        }
    }
    
    pub fn float(float : impl Into<f64>) -> Self {
        let f : f64 = float.into();
        Self::Float(f.to_bits())
    }
    
    pub fn int(integer : impl Into<i64>) -> Self {
        let i : i64 = integer.into();
        Self::Int(i)
    }
    
    pub fn uint(integer : impl Into<u64>) -> Self {
        let i : u64 = integer.into();
        Self::Uint(i)
    }
}

//==============================================================================================
//        Easy of use Traits
//==============================================================================================

pub trait IntoPakValue {
    fn into_pak_value(self) -> PakValue;
}

impl <T> IntoPakValue for Option<T> where T : IntoPakValue {
    fn into_pak_value(self) -> PakValue {
        match self {
            Some(value) => value.into_pak_value(),
            None => PakValue::Void,
        }
    }
}

impl <T> IntoPakValue for T where T : Into<PakValue> {
    fn into_pak_value(self) -> PakValue {
        self.into()
    }
}

impl<'s> From<&'s str> for PakValue {
    fn from(value: &'s str) -> Self {
        PakValue::String(value.to_string())
    }
}

impl From<String> for PakValue {
    fn from(value: String) -> Self {
        PakValue::String(value)
    }
}

impl From<f64> for PakValue {
    fn from(value: f64) -> Self {
        PakValue::Float(value.to_bits())
    }
}

impl From<f32> for PakValue {
    fn from(value: f32) -> Self {
        PakValue::Float((value as f64).to_bits())
    }
}

impl From<i64> for PakValue {
    fn from(value: i64) -> Self {
        PakValue::Int(value)
    }
}

impl From<i32> for PakValue {
    fn from(value: i32) -> Self {
        PakValue::Int(value as i64)
    }
}

impl From<i16> for PakValue {
    fn from(value: i16) -> Self {
        PakValue::Int(value as i64)
    }
}

impl From<i8> for PakValue {
    fn from(value: i8) -> Self {
        PakValue::Int(value as i64)
    }
}

impl From<u64> for PakValue {
    fn from(value: u64) -> Self {
        PakValue::Uint(value as u64)
    }
}

impl From<u32> for PakValue {
    fn from(value: u32) -> Self {
        PakValue::Uint(value as u64)
    }
}

impl From<u16> for PakValue {
    fn from(value: u16) -> Self {
        PakValue::Uint(value as u64)
    }
}

impl From<u8> for PakValue {
    fn from(value: u8) -> Self {
        PakValue::Uint(value as u64)
    }
}

impl From<bool> for PakValue {
    fn from(value: bool) -> Self {
        PakValue::Boolean(value)
    }
}