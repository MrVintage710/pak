use std::collections::HashSet;
use impl_trait_for_tuples::impl_for_tuples;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use crate::{error::PakResult, pointer::PakPointer, value::IntoPakValue, Pak, PakBuilder};
use super::index::PakIndex;

#[allow(unused_variables)]
pub trait PakItem where Self : Serialize + for<'de> Deserialize<'de> {
    fn pak(self, builder : &mut PakBuilder) -> PakResult<PakPointer> {
        let bytes = bincode::serialize(&self)?;
        let indices = self.indices();
        let pointer = builder.store::<Self>(bytes, indices)?;
        Ok(pointer)
    }
    
    fn unpak(pak : &Pak, pointer : &PakPointer) -> PakResult<Self> {
        pak.read_err(pointer)
    }
    
    fn prelude(&mut self, builder : &mut PakBuilder) -> PakResult<()> { Ok(()) }
    
    fn indices(&self) -> Vec<PakIndex>;
}

impl <T> PakItem for T where T : for<'de> Deserialize<'de> + Serialize + IntoPakValue {
    fn indices(&self) -> Vec<PakIndex> {
        Vec::new()
    }
}

impl <T> PakItem for Vec<T> where T : PakItem {
    
    fn prelude(&mut self, builder : &mut PakBuilder) -> PakResult<()> {
        for item in self.iter_mut() { item.prelude(builder)? }
        Ok(())
    }
    
    fn indices(&self) -> Vec<PakIndex> {
        self.iter().map(|item| item.indices()).flatten().collect()
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
//        IntoBytes
//==============================================================================================

pub trait IntoBytes {
    fn into_bytes(&self) -> PakResult<Vec<u8>>;
}

#[cfg(feature = "serde")]
impl <T> IntoBytes for T where T : Serialize {
    fn into_bytes(&self) -> PakResult<Vec<u8>> {
        bincode::serialize(self).map_err(|e| e.into())
    }
}

#[cfg(not(feature = "serde"))]
impl IntoBytes for Vec<Vec<u8>> {
    fn into_bytes(&mut self, _ : &mut PakBuilder) -> PakResult<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }
}

#[cfg(not(feature = "serde"))]
impl <T> IntoBytes for Vec<T> where T : IntoBytes {
    fn into_bytes(&mut self, builder : &mut PakBuilder) -> PakResult<Vec<u8>> {
        let mut bytes = vec![];
        for i in self {
            bytes.extend(i.into_bytes(builder)?);
        }
        Ok(bincode::serialize(&bytes)?)
    }
}

//==============================================================================================
//        FromBytes
//==============================================================================================

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

///This implementation is need for derive beheviors to 
#[cfg(not(feature = "serde"))]
#[cfg(feature = "derive")]
impl FromBytes for Vec<Vec<u8>> {
    fn from_bytes(bytes: &[u8]) -> PakResult<Self> {
        let obj : Self = bincode::deserialize::<Self>(bytes)?;
        Ok(obj)
    }
}


