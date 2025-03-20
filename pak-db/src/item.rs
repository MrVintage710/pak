use std::collections::HashSet;
use impl_trait_for_tuples::impl_for_tuples;
use serde::{de::DeserializeOwned, Serialize};
use crate::{error::PakResult, pointer::PakPointer, Pak, PakBuilder};
use super::index::PakIndex;

pub trait PakItem : Sized {
    fn pak(self, builder : &mut PakBuilder) -> PakResult<PakPointer>;
    
    fn unpak(pak : &Pak, pointer : &PakPointer) -> PakResult<Self>;
    
    fn indices(&self) -> Vec<PakIndex>;
}

impl <T> PakItem for T where T : IntoBytes + FromBytes + PakItemSearchable {
    fn indices(&self) -> Vec<PakIndex> {
        PakItemSearchable::get_indices(self)
    }

    fn pak(mut self, builder : &mut PakBuilder) -> PakResult<PakPointer> {
        let bytes = IntoBytes::into_bytes(&mut self, builder)?;
        let indices = self.indices();
        let pointer = builder.store::<Self>(bytes, indices)?;
        Ok(pointer)
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

impl <T> IntoBytes for PakItemRef<T> where T : PakItem {
    fn into_bytes(&mut self, builder : &mut PakBuilder) -> PakResult<Vec<u8>> {
        self.store(builder)?;
        self.unwrap_pointer().into_bytes(builder)
    }
}

impl <T> FromBytes for PakItemRef<T> where T : PakItem {
    fn from_bytes(bytes : &[u8]) -> PakResult<Self> {
        let pointer = PakPointer::from_bytes(bytes)?;
        Ok(PakItemRef::Ref(pointer))
    }
}

impl <T> PakItemSearchable for PakItemRef<T> where T : PakItem {
    fn get_indices(&self) -> Vec<PakIndex> {
        match self {
            PakItemRef::Ref(_) => Vec::new(),
            PakItemRef::Loaded(item) => item.indices(),
        }
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

//==============================================================================================
//        IntoBytes
//==============================================================================================

pub trait IntoBytes {
    fn into_bytes(&mut self, builder : &mut PakBuilder) -> PakResult<Vec<u8>>;
}

#[cfg(feature = "serde")]
impl <T> IntoBytes for T where T : Serialize {
    fn into_bytes(&mut self, _ : &mut PakBuilder) -> PakResult<Vec<u8>> {
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

//==============================================================================================
//        PakItemSearchable
//==============================================================================================

pub trait PakItemSearchable {
    fn get_indices(&self) -> Vec<PakIndex>;
}


