use std::marker::PhantomData;

use deserialize::PakItemRefVisitor;
use serde::{de::{VariantAccess, Visitor}, Deserialize, Serialize};

use crate::{error::PakResult, index::PakIndex, item::PakItem, pointer::PakPointer, Pak, PakBuilder};


//==============================================================================================
//        PakItemRef
//==============================================================================================

#[derive(Debug, Serialize, Clone)]
pub enum PakItemRef<T> where T : PakItem {
    Ref(PakPointer),
    Loaded(T),
}

impl <T> PakItem for PakItemRef<T> where T : PakItem {
    fn indices(&self) -> Vec<PakIndex> {
        match self {
            PakItemRef::Ref(_) => vec![],
            PakItemRef::Loaded(item) => item.indices(),
        }
    }
    
    fn prelude(&mut self, builder : &mut PakBuilder) -> PakResult<()> {
        self.store(builder)
    }
}

impl <'de, T> Deserialize<'de> for PakItemRef<T> where T : PakItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        deserializer.deserialize_enum("PakItemRef", &["Ref", "Loaded"], PakItemRefVisitor(PhantomData))
    }
}

impl <T> PakItemRef<T> where T : PakItem {
    
    pub fn pointer(pointer : PakPointer) -> Self {
        PakItemRef::Ref(pointer)
    }
    
    pub fn load(&mut self, pak : &Pak) -> PakResult<()> {
        match self {
            PakItemRef::Ref(pointer) => {
                let item = pak.unpak_err::<T>(pointer)?;
                *self = PakItemRef::Loaded(item);
                Ok(())
            },
            PakItemRef::Loaded(_) => Ok(())
        }
    }
    
    pub fn store(&mut self, builder : &mut PakBuilder) -> PakResult<()> {
        if self.is_ref() { return Ok(()) };
        let old = std::mem::replace(self, PakItemRef::pointer(PakPointer::default()));
        let item = old.unwrap();
        let pointer = builder.pak(item)?;
        *self = PakItemRef::Ref(pointer);
        Ok(())
    }
    
    pub fn get(&self) -> &T {
        match self {
            PakItemRef::Ref(_) => panic!("Tried to get a value of Ref"),
            PakItemRef::Loaded(item) => item,
        }
    }
    
    pub fn get_mut(&mut self) -> &mut T {
        match self {
            PakItemRef::Ref(_) => panic!("Tried to get a value of Ref"),
            PakItemRef::Loaded(item) => item,
        }
    }
    
    pub fn unwrap(self) -> T {
        match self {
            PakItemRef::Ref(_) => panic!("Tried to get a value of Ref"),
            PakItemRef::Loaded(item) => item,
        }
    }
    
    pub fn unwrap_pointer(&self) -> &PakPointer {
        match self {
            PakItemRef::Ref(pointer) => pointer,
            PakItemRef::Loaded(_) => panic!("Tried to get a pointer of Loaded"),
        }
    }
    
    pub fn unwrap_or_load(self, pak : &Pak) -> PakResult<T> {
        match self {
            PakItemRef::Ref(pointer) => {
                let item = pak.unpak_err::<T>(&pointer)?;
                Ok(item)
            },
            PakItemRef::Loaded(item) => Ok(item),
        }
    }
    
    pub fn is_loaded(&self) -> bool {
        match self {
            PakItemRef::Ref(_) => false,
            PakItemRef::Loaded(_) => true,
        }
    }
    
    pub fn is_ref(&self) -> bool {
        match self {
            PakItemRef::Ref(_) => true,
            PakItemRef::Loaded(_) => false,
        }
    }
}

#[doc(hidden)]
mod deserialize {
    use super::*;
    
    pub struct PakItemRefVisitor<T>(pub PhantomData<T>) where T : PakItem;
    
    impl <'de, T> Visitor<'de> for PakItemRefVisitor<T> where T : PakItem {
        type Value = PakItemRef<T>;
    
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("A reference to a PakItem")
        }
        
        fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error> where A: serde::de::EnumAccess<'de> {
            match data.variant()? {
                (PakItemRefField::Ref, variant) => {
                    let value = variant.newtype_variant::<PakPointer>()?;
                    Ok(PakItemRef::Ref(value))
                }
                (PakItemRefField::Loaded, variant) => {
                    let value = variant.newtype_variant::<T>()?;
                    Ok(PakItemRef::Loaded(value))
                },
            }
        }
    }
    
    #[derive(Deserialize, Serialize)]
    enum PakItemRefField {
        Ref,
        Loaded,
    }
}


