use std::collections::HashSet;

use serde::{de::DeserializeOwned, Serialize};
use crate::{error::PakResult, Pak};
use super::{index::PakIndex, PakPointer};

//==============================================================================================
//        PakItem Traits
//==============================================================================================

pub trait PakItemSearchable {
    fn get_indices(&self) -> Vec<PakIndex>;
}

pub trait PakItemSerialize {
    fn into_bytes(&self) -> PakResult<Vec<u8>>;
}

pub trait PakItemDeserialize: Sized {
    fn from_bytes(bytes: &[u8]) -> PakResult<Self>;
    
    fn from_pak(pak : &[u8], pointer : PakPointer) -> PakResult<Self> {
        let data = &pak[pointer.offset as usize..pointer.offset as usize + pointer.size as usize];
        let res = Self::from_bytes(data)?;
        Ok(res)
    }
}

impl <T> PakItemDeserialize for T where T : DeserializeOwned {
    fn from_bytes(bytes: &[u8]) -> PakResult<Self> {
        let obj : Self = bincode::deserialize::<Self>(bytes)?;
        Ok(obj)
    }
}

impl <T> PakItemSerialize for T where T : Serialize {
    fn into_bytes(&self) -> PakResult<Vec<u8>> {
        bincode::serialize(self).map_err(|e| e.into())
    }
}

//==============================================================================================
//        PakItemDeserialzedGroup
//==============================================================================================

pub trait PakItemDeserializeGroup {
    type ReturnType;
    
    fn deserialize_group(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType>;
}

impl <T> PakItemDeserializeGroup for (T, ) where T : PakItemDeserialize {
    type ReturnType = Vec<T>;
    
    fn deserialize_group(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType> {
        let values = pointers.into_iter().filter_map(|pointer| pak.read::<T>(pointer)).collect::<Vec<_>>();
        Ok(values)
    }
}

impl <T1, T2> PakItemDeserializeGroup for (T1, T2) where T1 : PakItemDeserialize, T2 : PakItemDeserialize {
    type ReturnType = (Vec<T1>, Vec<T2>);

    fn deserialize_group(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType> {
        let t1 = pointers.iter().filter_map(|pointer| pak.read::<T1>(*pointer)).collect::<Vec<_>>();
        let t2 = pointers.iter().filter_map(|pointer| pak.read::<T2>(*pointer)).collect::<Vec<_>>();
        return Ok((t1, t2));
    }
}