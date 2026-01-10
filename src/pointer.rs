use ordermap::OrderSet;
use serde::{Deserialize, Serialize};

use crate::{Pak, error::PakResult, item::{PakItemDeserialize, PakItemDeserializeGroup}, query::PakQueryExpression};

//==============================================================================================
//        PakPointer
//==============================================================================================

/// A pointer that points to a specific location in the pak file. It comes in two flavors, typed and untyped. This pointer is typically offset by the size of the header.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum PakPointer {
    Typed(PakTypedPointer),
    Untyped(PakUntypedPointer),
}

impl PakPointer {
    pub fn new_typed<T>(offset : u64, size : u64) -> Self {
        Self::Typed(PakTypedPointer::new(offset, size, std::any::type_name::<T>()))
    }

    pub fn new_untyped(offset : u64, size : u64) -> Self {
        Self::Untyped(PakUntypedPointer::new(offset, size))
    }
    
    pub fn offset(&self) -> u64 {
        match self {
            Self::Typed(ptr) => ptr.offset,
            Self::Untyped(ptr) => ptr.offset,
        }
    }
    
    pub fn size(&self) -> u64 {
        match self {
            Self::Typed(ptr) => ptr.size,
            Self::Untyped(ptr) => ptr.size,
        }
    }
    
    pub fn type_name(&self) -> &str {
        match self {
            Self::Typed(ptr) => &ptr.type_name,
            Self::Untyped(_) => "Untyped",
        }
    }
    
    pub fn as_untyped(&self) -> PakUntypedPointer {
        match self {
            Self::Typed(ptr) => PakUntypedPointer::new(ptr.offset, ptr.size),
            Self::Untyped(ptr) => *ptr,
        }
    }
    
    pub fn into_typed<T>(self) -> PakTypedPointer {
        match self {
            Self::Typed(ptr) => ptr,
            Self::Untyped(ptr) => PakTypedPointer::new(ptr.offset, ptr.size, std::any::type_name::<T>()),
        }
    }
    
    pub fn type_is_match<T>(&self) -> bool {
        match self {
            Self::Typed(ptr) => ptr.type_name == std::any::type_name::<T>(),
            Self::Untyped(_) => true,
        }
    }
    
    pub fn drop_type(&mut self) {
        let pointer = PakPointer::Untyped(PakUntypedPointer { offset: self.offset(), size: self.size() });
        *self = pointer
    }
}

impl Clone for PakPointer {
    fn clone(&self) -> Self {
        match self {
            Self::Typed(ptr) => Self::Typed(ptr.clone()),
            Self::Untyped(ptr) => Self::Untyped(*ptr),
        }
    }
}

impl <T> PakQueryExpression<T> for PakPointer where T : PakItemDeserializeGroup {
    fn execute(&self, _pak : &crate::Pak) -> PakResult<OrderSet<PakPointer>> {
        Ok(OrderSet::from([self.clone()]))
    }
}

impl From<PakUntypedPointer> for PakPointer {
    fn from(value: PakUntypedPointer) -> Self {
        PakPointer::Untyped(value)
    }
}

impl From<PakTypedPointer> for PakPointer {
    fn from(value: PakTypedPointer) -> Self {
        PakPointer::Typed(value)
    }
}

//==============================================================================================
//        PakTypedPointer
//==============================================================================================

/// A typed pointer. This tells you what rust type is stored at the location pointed to. You can check it with a type at runtime to fail requests that have a type mismatch.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize, Hash)]
pub struct PakTypedPointer {
    offset : u64,
    size : u64,
    type_name : String,
}

impl PakTypedPointer {
    pub fn new(offset : u64, size : u64, type_name : &str) -> Self {
        Self { offset, size, type_name : type_name.to_string() }
    }
    
    pub fn into_pointer(self) -> PakPointer {
        PakPointer::Typed(self)
    }
}

//==============================================================================================
//        PakUntypedPointer
//==============================================================================================

/// An untyped pointer. This tells you the offset and size of the data at the location pointed to. This is useful if you always know the type of the data at the location pointed to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize, Hash)]
pub struct PakUntypedPointer {
    offset : u64,
    size : u64,
}

impl PakUntypedPointer {
    pub fn new(offset : u64, size : u64) -> Self {
        Self { offset, size }
    }
    
    pub fn as_pointer(&self) -> PakPointer {
        PakPointer::Untyped(*self)
    }
}

//==============================================================================================
//        PakCache
//==============================================================================================

pub struct PakCache<T> where T : PakItemDeserialize {
    pointer : PakPointer,
    cache : Option<T>
}

impl <T> PakCache<T> where T : PakItemDeserialize {
    
    pub fn fetch(&mut self, pak : &Pak) -> PakResult<&T> {
        if self.cache.is_some() {
            Ok(self.cache.as_ref().unwrap())
        } else {
            let value = pak.read_err::<T>(&self.pointer)?;
            self.cache = Some(value);
            Ok(self.cache.as_ref().unwrap())
        }
    }
    
    pub fn fetch_mut(&mut self, pak : &Pak) -> PakResult<&mut T> {
        if self.cache.is_some() {
            Ok(self.cache.as_mut().unwrap())
        } else {
            let value = pak.read_err::<T>(&self.pointer)?;
            self.cache = Some(value);
            Ok(self.cache.as_mut().unwrap())
        }
    }
    
    pub fn unwrap(self) -> T {
        self.cache.unwrap()
    }
    
    pub fn get(&self) -> Option<&T> {
        self.cache.as_ref()
    }
    
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.cache.as_mut()
    }
}

impl <T> From<PakPointer> for PakCache<T> where T : PakItemDeserialize {
    fn from(value: PakPointer) -> Self {
        PakCache { pointer: value, cache: None }
    }
}

impl <T> From<PakUntypedPointer> for PakCache<T> where for<'de> T : Deserialize<'de> {
    fn from(value: PakUntypedPointer) -> Self {
        let pointer : PakPointer = value.into();
        pointer.into()
    }
}

impl <T> From<PakTypedPointer> for PakCache<T> where for<'de> T : Deserialize<'de> {
    fn from(value: PakTypedPointer) -> Self {
        let pointer : PakPointer = value.into();
        pointer.into()
    }
}