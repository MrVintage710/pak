use std::collections::HashSet;

use impl_trait_for_tuples::impl_for_tuples;
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

impl <T> PakItemDeserializeGroup for (T, ) where T : PakItemDeserialize{
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

impl <T1, T2, T3> PakItemDeserializeGroup for (T1, T2, T3) where T1 : PakItemDeserialize, T2 : PakItemDeserialize, T3 : PakItemDeserialize {
    type ReturnType = (Vec<T1>, Vec<T2>, Vec<T3>);

    fn deserialize_group(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType> {
        let t1 = pointers.iter().filter_map(|pointer| pak.read::<T1>(*pointer)).collect::<Vec<_>>();
        let t2 = pointers.iter().filter_map(|pointer| pak.read::<T2>(*pointer)).collect::<Vec<_>>();
        let t3 = pointers.iter().filter_map(|pointer| pak.read::<T3>(*pointer)).collect::<Vec<_>>();
        return Ok((t1, t2, t3));
    }
}

impl <T1, T2, T3, T4> PakItemDeserializeGroup for (T1, T2, T3, T4) where T1 : PakItemDeserialize, T2 : PakItemDeserialize, T3 : PakItemDeserialize, T4 : PakItemDeserialize {
    type ReturnType = (Vec<T1>, Vec<T2>, Vec<T3>, Vec<T4>);

    fn deserialize_group(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType> {
        let t1 = pointers.iter().filter_map(|pointer| pak.read::<T1>(*pointer)).collect::<Vec<_>>();
        let t2 = pointers.iter().filter_map(|pointer| pak.read::<T2>(*pointer)).collect::<Vec<_>>();
        let t3 = pointers.iter().filter_map(|pointer| pak.read::<T3>(*pointer)).collect::<Vec<_>>();
        let t4 = pointers.iter().filter_map(|pointer| pak.read::<T4>(*pointer)).collect::<Vec<_>>();
        return Ok((t1, t2, t3, t4));
    }
}

impl <T1, T2, T3, T4, T5> PakItemDeserializeGroup for (T1, T2, T3, T4, T5) where T1 : PakItemDeserialize, T2 : PakItemDeserialize, T3 : PakItemDeserialize, T4 : PakItemDeserialize, T5 : PakItemDeserialize {
    type ReturnType = (Vec<T1>, Vec<T2>, Vec<T3>, Vec<T4>, Vec<T5>);

    fn deserialize_group(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType> {
        let t1 = pointers.iter().filter_map(|pointer| pak.read::<T1>(*pointer)).collect::<Vec<_>>();
        let t2 = pointers.iter().filter_map(|pointer| pak.read::<T2>(*pointer)).collect::<Vec<_>>();
        let t3 = pointers.iter().filter_map(|pointer| pak.read::<T3>(*pointer)).collect::<Vec<_>>();
        let t4 = pointers.iter().filter_map(|pointer| pak.read::<T4>(*pointer)).collect::<Vec<_>>();
        let t5 = pointers.iter().filter_map(|pointer| pak.read::<T5>(*pointer)).collect::<Vec<_>>();
        return Ok((t1, t2, t3, t4, t5));
    }
}

impl <T1, T2, T3, T4, T5, T6> PakItemDeserializeGroup for (T1, T2, T3, T4, T5, T6) where T1 : PakItemDeserialize, T2 : PakItemDeserialize, T3 : PakItemDeserialize, T4 : PakItemDeserialize, T5 : PakItemDeserialize, T6 : PakItemDeserialize {
    type ReturnType = (Vec<T1>, Vec<T2>, Vec<T3>, Vec<T4>, Vec<T5>, Vec<T6>);

    fn deserialize_group(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType> {
        let t1 = pointers.iter().filter_map(|pointer| pak.read::<T1>(*pointer)).collect::<Vec<_>>();
        let t2 = pointers.iter().filter_map(|pointer| pak.read::<T2>(*pointer)).collect::<Vec<_>>();
        let t3 = pointers.iter().filter_map(|pointer| pak.read::<T3>(*pointer)).collect::<Vec<_>>();
        let t4 = pointers.iter().filter_map(|pointer| pak.read::<T4>(*pointer)).collect::<Vec<_>>();
        let t5 = pointers.iter().filter_map(|pointer| pak.read::<T5>(*pointer)).collect::<Vec<_>>();
        let t6 = pointers.iter().filter_map(|pointer| pak.read::<T6>(*pointer)).collect::<Vec<_>>();
        return Ok((t1, t2, t3, t4, t5, t6));
    }
}

impl <T1, T2, T3, T4, T5, T6, T7> PakItemDeserializeGroup for (T1, T2, T3, T4, T5, T6, T7) where T1 : PakItemDeserialize, T2 : PakItemDeserialize, T3 : PakItemDeserialize, T4 : PakItemDeserialize, T5 : PakItemDeserialize, T6 : PakItemDeserialize, T7 : PakItemDeserialize {
    type ReturnType = (Vec<T1>, Vec<T2>, Vec<T3>, Vec<T4>, Vec<T5>, Vec<T6>, Vec<T7>);

    fn deserialize_group(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType> {
        let t1 = pointers.iter().filter_map(|pointer| pak.read::<T1>(*pointer)).collect::<Vec<_>>();
        let t2 = pointers.iter().filter_map(|pointer| pak.read::<T2>(*pointer)).collect::<Vec<_>>();
        let t3 = pointers.iter().filter_map(|pointer| pak.read::<T3>(*pointer)).collect::<Vec<_>>();
        let t4 = pointers.iter().filter_map(|pointer| pak.read::<T4>(*pointer)).collect::<Vec<_>>();
        let t5 = pointers.iter().filter_map(|pointer| pak.read::<T5>(*pointer)).collect::<Vec<_>>();
        let t6 = pointers.iter().filter_map(|pointer| pak.read::<T6>(*pointer)).collect::<Vec<_>>();
        let t7 = pointers.iter().filter_map(|pointer| pak.read::<T7>(*pointer)).collect::<Vec<_>>();
        return Ok((t1, t2, t3, t4, t5, t6, t7));
    }
}

impl <T1, T2, T3, T4, T5, T6, T7, T8> PakItemDeserializeGroup for (T1, T2, T3, T4, T5, T6, T7, T8) where T1 : PakItemDeserialize, T2 : PakItemDeserialize, T3 : PakItemDeserialize, T4 : PakItemDeserialize, T5 : PakItemDeserialize, T6 : PakItemDeserialize, T7 : PakItemDeserialize, T8 : PakItemDeserialize {
    type ReturnType = (Vec<T1>, Vec<T2>, Vec<T3>, Vec<T4>, Vec<T5>, Vec<T6>, Vec<T7>, Vec<T8>);

    fn deserialize_group(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType> {
        let t1 = pointers.iter().filter_map(|pointer| pak.read::<T1>(*pointer)).collect::<Vec<_>>();
        let t2 = pointers.iter().filter_map(|pointer| pak.read::<T2>(*pointer)).collect::<Vec<_>>();
        let t3 = pointers.iter().filter_map(|pointer| pak.read::<T3>(*pointer)).collect::<Vec<_>>();
        let t4 = pointers.iter().filter_map(|pointer| pak.read::<T4>(*pointer)).collect::<Vec<_>>();
        let t5 = pointers.iter().filter_map(|pointer| pak.read::<T5>(*pointer)).collect::<Vec<_>>();
        let t6 = pointers.iter().filter_map(|pointer| pak.read::<T6>(*pointer)).collect::<Vec<_>>();
        let t7 = pointers.iter().filter_map(|pointer| pak.read::<T7>(*pointer)).collect::<Vec<_>>();
        let t8 = pointers.iter().filter_map(|pointer| pak.read::<T8>(*pointer)).collect::<Vec<_>>();
        return Ok((t1, t2, t3, t4, t5, t6, t7, t8));
    }
}