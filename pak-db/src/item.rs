use std::collections::HashSet;
use impl_trait_for_tuples::impl_for_tuples;
use serde::{de::DeserializeOwned, Serialize};
use crate::{error::PakResult, pointer::PakPointer, Pak, PakBuilder};
use super::index::PakIndex;

pub trait PakItem : Sized {
    fn pak(&mut self, builder : &mut PakBuilder) -> PakResult<()>;
    
    fn unpak(pak : &Pak, pointer : &PakPointer) -> PakResult<Self>;
    
    fn indices(&self) -> Vec<PakIndex>;
}

impl <T> PakItem for T where T : IntoBytes + FromBytes + PakItemSearchable {
    fn indices(&self) -> Vec<PakIndex> {
        PakItemSearchable::get_indices(self)
    }

    fn pak(&mut self, builder : &mut PakBuilder) -> PakResult<()> {
        let bytes = IntoBytes::into_bytes(self)?;
        let indices = self.indices();
        builder.store::<Self>(bytes, indices)?;
        Ok(())
    }

    fn unpak(pak : &Pak, pointer : &PakPointer) -> PakResult<Self> {
        let bytes = pak.read_bytes(pointer)?;
        FromBytes::from_bytes(&bytes)
    }
}

//==============================================================================================
//        PakItemGroup
//==============================================================================================

pub trait PakItemGroup {
    type ReturnType;
    
    fn collect(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType>;
}

impl <T> PakItemGroup for T where T : PakItem {
    type ReturnType = Vec<T>;
    
    fn collect(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType> {
        let set = pointers.iter().filter_map(|pointer| pak.unpak::<T>(pointer).map(|inner| Some(inner)).unwrap_or(None)).collect::<Vec<_>>();
        Ok(set)
    }
}

#[impl_for_tuples(12)]
impl PakItemGroup for Tuple {
    for_tuples!(type ReturnType = (#( Tuple::ReturnType ),*););
    
    fn collect(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType> {
        Ok(for_tuples!((#(Tuple::collect(pak, pointers.clone())?),*)))
    }
}

//==============================================================================================
//        PakItemRef
//==============================================================================================

pub enum PakItemRef<T> where T : PakItem {
    Ref(PakPointer),
    Loaded(T),
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
    
    pub fn unwrap_or_load(self, pak : &Pak) -> PakResult<T> {
        match self {
            PakItemRef::Ref(pointer) => {
                let item = pak.unpak_err::<T>(&pointer)?;
                Ok(item)
            },
            PakItemRef::Loaded(item) => Ok(item),
        }
    }
}

//==============================================================================================
//        PakItem Traits
//==============================================================================================

pub trait PakItemSearchable {
    fn get_indices(&self) -> Vec<PakIndex>;
}

pub trait IntoBytes {
    fn into_bytes(&self) -> PakResult<Vec<u8>>;
}

pub trait FromBytes: Sized {
    fn from_bytes(bytes: &[u8]) -> PakResult<Self>;
}

#[cfg(feature = "serde")]
impl <T> FromBytes for T where T : DeserializeOwned {
    fn from_bytes(bytes: &[u8]) -> PakResult<Self> {
        let obj : Self = bincode::deserialize::<Self>(bytes)?;
        Ok(obj)
    }
}

#[cfg(feature = "serde")]
impl <T> IntoBytes for T where T : Serialize {
    fn into_bytes(&self) -> PakResult<Vec<u8>> {
        bincode::serialize(self).map_err(|e| e.into())
    }
}