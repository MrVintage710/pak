use std::collections::HashSet;
use serde::{Deserialize};
use crate::{error::PakResult, pointer::PakPointer, Pak};
use super::index::PakIndex;

//==============================================================================================
//        PakItem Traits
//==============================================================================================

pub trait PakSearchable {
    fn get_indices(&self) -> Vec<PakIndex>;
}

//==============================================================================================
//        PakItemDeserialzedGroup
//==============================================================================================

pub trait DeserializeGroup {
    type ReturnType;
    
    fn deserialize_group(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType>;
    
    fn get_types() -> Vec<&'static str>;
}

macro_rules! impl_group {
    ( $( $name:ident )+ ) => {
        impl <$($name,)+> DeserializeGroup for ($($name, )+ ) where $($name : for<'de> Deserialize<'de>, )+ {
            type ReturnType = ($(Vec<$name>, )+);
            
            fn deserialize_group(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType> {
                Ok(($(pointers.iter().filter_map(|pointer| pak.read::<$name>(pointer)).collect::<Vec<_>>(),)+))
            }
            
            fn get_types() -> Vec<&'static str> {
                vec![$(std::any::type_name::<$name>(), )+]
            }
        }
    };
}

impl <T> DeserializeGroup for (T, ) where T : for<'de> Deserialize<'de> {
    type ReturnType = Vec<T>;
    
    fn deserialize_group(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType> {
        Ok(pointers.iter().filter_map(|pointer| pak.read::<T>(pointer)).collect::<Vec<_>>())
    }
    
    fn get_types() -> Vec<&'static str> {
        vec![
            std::any::type_name::<T>()
        ]
    }
}

impl_group!{ A B }
impl_group!{ A B C }
impl_group!{ A B C D }
impl_group!{ A B C D E }
impl_group!{ A B C D E F }
impl_group!{ A B C D E F G }
impl_group!{ A B C D E F G H }
impl_group!{ A B C D E F G H I }
impl_group!{ A B C D E F G H I J }
impl_group!{ A B C D E F G H I J K }