//==============================================================================================
//        Pak Values
//==============================================================================================

use std::fmt::Debug;
use serde::{Deserialize, Serialize};

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
//        Ease of use Traits
//==============================================================================================

pub trait IntoPakValue {
    fn into_pak_value(&self) -> PakValue;
}

impl <T> IntoPakValue for Option<T> where T : IntoPakValue {
    fn into_pak_value(&self) -> PakValue {
        match self {
            Some(value) => value.into_pak_value(),
            None => PakValue::Void,
        }
    }
}

impl <T> IntoPakValue for &T where T : IntoPakValue {
    fn into_pak_value(&self) -> PakValue {
        (*self).into_pak_value()
    }
}

impl <T> IntoPakValue for &mut T where T : IntoPakValue {
    fn into_pak_value(&self) -> PakValue {
        (**self).into_pak_value()
    }
}

impl IntoPakValue for &str {
    fn into_pak_value(&self) -> PakValue {
        PakValue::String(self.to_string())
    }
}

impl IntoPakValue for String {
    fn into_pak_value(&self) -> PakValue {
        PakValue::String(self.clone())
    }
}

impl IntoPakValue for f64 {
    fn into_pak_value(&self) -> PakValue {
        PakValue::Float(self.to_bits())
    }
}

impl IntoPakValue for f32 {
    fn into_pak_value(&self) -> PakValue {
        PakValue::Float((*self as f64).to_bits())
    }
}

impl IntoPakValue for i64 {
    fn into_pak_value(&self) -> PakValue {
        PakValue::Int(*self)
    }
}

impl IntoPakValue for i32 {
    fn into_pak_value(&self) -> PakValue {
        PakValue::Int(*self as i64)
    }
}

impl IntoPakValue for i16 {
    fn into_pak_value(&self) -> PakValue {
        PakValue::Int(*self as i64)
    }
}

impl IntoPakValue for i8 {
    fn into_pak_value(&self) -> PakValue {
        PakValue::Int(*self as i64)
    }
}

impl IntoPakValue for u64 {
    fn into_pak_value(&self) -> PakValue {
        PakValue::Uint(*self)
    }
}

impl IntoPakValue for u32 {
    fn into_pak_value(&self) -> PakValue {
        PakValue::Uint(*self as u64)
    }
}

impl IntoPakValue for u16 {
    fn into_pak_value(&self) -> PakValue {
        PakValue::Uint(*self as u64)
    }
}

impl IntoPakValue for u8 {
    fn into_pak_value(&self) -> PakValue {
        PakValue::Uint(*self as u64)
    }
}

impl IntoPakValue for bool {
    fn into_pak_value(&self) -> PakValue {
        PakValue::Boolean(*self)
    }
}