use std::{marker::PhantomData};
use ordermap::OrderSet;
use serde::{Deserialize};
use crate::{Pak, error::PakResult, item::PakDeserialize, pointer::PakPointer};

//==============================================================================================
//        DeseerializeUnit
//==============================================================================================

pub trait DeserializeUnit {
    type ReturnType;
    
     fn deserialize_unit(pak : &Pak, pointer : &PakPointer) -> PakResult<Self::ReturnType>;
     
     fn type_name() -> &'static str;
}

impl <T> DeserializeUnit for T where T : PakDeserialize {
    type ReturnType = T;

    fn deserialize_unit(pak : &Pak, pointer : &PakPointer) -> PakResult<Self::ReturnType> {
        let result =  pak.read::<T>(&pointer)?;
        Ok(result)
    }

    fn type_name() -> &'static str {
        std::any::type_name::<T>()
    }
}

//==============================================================================================
//        PakItemDeserialzedGroup
//==============================================================================================

pub trait DeserializeGroup {
    type ReturnType;
    
    fn deserialize_group(pak : &Pak, pointers : OrderSet<PakPointer>) -> PakResult<Self::ReturnType>;
    
    fn get_types() -> Vec<&'static str>;
}

impl <T> DeserializeGroup for (T, ) where T : DeserializeUnit {
    type ReturnType = Vec<T::ReturnType>;
    
    fn deserialize_group(pak : &Pak, pointers : OrderSet<PakPointer>) -> PakResult<Self::ReturnType> {
        Ok(pointers.iter().filter_map(|pointer| T::deserialize_unit(pak, pointer).ok()).collect::<Vec<_>>())
    }
    
    fn get_types() -> Vec<&'static str> {
        vec![
            T::type_name()
        ]
    }
}

macro_rules! impl_group {
    ( $( $name:ident )+ ) => {
        impl <$($name,)+> DeserializeGroup for ($($name, )+ ) where $($name : DeserializeUnit, )+ {
            type ReturnType = ($(Vec<$name::ReturnType>, )+);
            
            fn deserialize_group(pak : &Pak, pointers : OrderSet<PakPointer>) -> PakResult<Self::ReturnType> {
                Ok(($(pointers.iter().filter_map(|pointer| $name::deserialize_unit(pak, pointer).ok()).collect::<Vec<_>>(),)+))
            }
            
            fn get_types() -> Vec<&'static str> {
                vec![$($name::type_name(), )+]
            }
        }
    };
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

//==============================================================================================
//        Pointer
//==============================================================================================

pub struct Pointer<T>(PhantomData<T>) where T : for <'de> Deserialize<'de>;

impl <T> DeserializeUnit for Pointer<T> where T : for<'de> Deserialize<'de> {
    type ReturnType = PakPointer;

    fn deserialize_unit(_pak : &Pak, pointer : &PakPointer) -> PakResult<Self::ReturnType> {
        pointer.check_type::<T>()?;
        Ok(pointer.clone())
    }

    fn type_name() -> &'static str {
        std::any::type_name::<T>()
    }
}