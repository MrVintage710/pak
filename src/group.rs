use std::{collections::HashSet, marker::PhantomData, sync::Weak};
use serde::{Deserialize};
use crate::{Pak, PakInner, error::{PakError, PakResult}, pointer::PakPointer};

//==============================================================================================
//        DeseerializeUnit
//==============================================================================================

pub trait DeserializeUnit {
    type ReturnType;
    
     fn deserialize_unit(pak : &Pak, pointer : &PakPointer) -> PakResult<Self::ReturnType>;
     
     fn type_name() -> &'static str;
}

impl <T> DeserializeUnit for T where T : for<'de> Deserialize<'de> {
    type ReturnType = T;

    fn deserialize_unit(pak : &Pak, pointer : &PakPointer) -> PakResult<Self::ReturnType> {
        let result =  pak.read_err::<T>(&pointer)?;
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
    
    fn deserialize_group(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType>;
    
    fn get_types() -> Vec<&'static str>;
}

impl <T> DeserializeGroup for (T, ) where T : DeserializeUnit {
    type ReturnType = Vec<T::ReturnType>;
    
    fn deserialize_group(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType> {
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
            
            fn deserialize_group(pak : &Pak, pointers : HashSet<PakPointer>) -> PakResult<Self::ReturnType> {
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
//        Defer
//==============================================================================================

/// This is a query result that is defered for later. This is useful when you have a lot of data that you need to load,
/// but you are okay loading each later.
pub struct Defer<T> where T : for<'de> Deserialize<'de> {
    pak : Weak<PakInner>,
    pointer : PakPointer,
    data : Option<T>
}

impl <T> Defer<T> where T : for<'de> Deserialize<'de> {
    pub fn new(pak : &Pak, pointer : PakPointer) -> Defer<T> {
        Defer { pak : pak.weak(), pointer, data: None }
    }
    
    pub fn get(&mut self) -> PakResult<&T> {
        if let None = self.data {
            let Some(pak) = self.pak.upgrade() else { return Err(PakError::PakDropped) };
            let value = pak.read_err::<T>(&self.pointer)?;
            self.data = Some(value);
        }
        Ok(self.data.as_ref().unwrap())
    }
    
    pub fn get_mut(&mut self) -> PakResult<&mut T> {
        if let None = self.data {
            let Some(pak) = self.pak.upgrade() else { return Err(PakError::PakDropped) };
            let value = pak.read_err::<T>(&self.pointer)?;
            self.data = Some(value);
        }
        Ok(self.data.as_mut().unwrap())
    }
    
    pub fn peek(&self) -> Option<&T> {
        self.data.as_ref()
    }
    
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.data.as_mut()
    }
    
    pub fn is_loaded(&self) -> bool {
        self.data.is_some()
    }
}

impl <T> DeserializeUnit for Defer<T> where T : for<'de> Deserialize<'de> {
    type ReturnType = Defer<T>;

    fn deserialize_unit(pak : &Pak, pointer : &PakPointer) -> PakResult<Self::ReturnType> {
        pointer.check_type::<T>()?;
        Ok(Defer::new(pak, pointer.clone()))
    }

    fn type_name() -> &'static str {
        std::any::type_name::<T>()
    }
}

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