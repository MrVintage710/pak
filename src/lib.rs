#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/MrVintage710/pak/refs/heads/main/docs/icon.png")]

use std::{collections::HashMap, fs::File, io::{BufReader, Read, Seek, SeekFrom}, path::Path, sync::{Arc, RwLock, Weak}};
use btree::PakTree;
use group::{DeserializeGroup};
use meta::{PakMeta, PakSizing};
use ordermap::OrderSet;
use pointer::{PakPointer, PakUntypedPointer};
use query::PakQueryExpression;
use serde::Deserialize;

use crate::{error::PakResult, item::PakDeserialize, pointer::PakTypedPointer};

#[cfg(test)]
mod test;

pub mod meta;
pub mod group;
pub mod index;
pub mod value;
pub(crate) mod btree;
pub mod query;
pub mod error;
pub mod pointer;
pub mod builder;
pub mod item;

//==============================================================================================
//        Pak File
//==============================================================================================

pub const PAK_FILE_VERSION : &'static str = "1.1";

pub const PAK_SIZING_STRUCT_SIZE_IN_BYTES : u64 = 32;

/// Represents a Pak file. This struct provides access to the metadata and data stored within the Pak file.
pub struct Pak {
    inner : Arc<PakInner>
}

impl Pak {
    /// Creates a new Pak instance from a [PakSource](crate::PakSource).
    pub fn new<S>(mut source : S) -> PakResult<Self> where S : PakSource + Send + Sync + 'static {
        let sizing_pointer = PakPointer::new_untyped(0, PAK_SIZING_STRUCT_SIZE_IN_BYTES);
        let sizing_buffer = source.read(&sizing_pointer, 0)?;
        let sizing : PakSizing = bincode::deserialize(&sizing_buffer)?;
        
        let meta_pointer = PakPointer::new_untyped(PAK_SIZING_STRUCT_SIZE_IN_BYTES, sizing.meta_size);
        let meta_buffer = source.read(&meta_pointer, 0)?;
        let meta : PakMeta = bincode::deserialize(&meta_buffer)?;
        
        let inner = Arc::new(PakInner { sizing, source : RwLock::new(Box::new(source)), meta });

        Ok(Self { inner })
    }
    
    /// Loads a Pak from the specified file path. This will not load the entire pak file into memory, just the header.
    pub fn new_from_file<P>(path : P) -> PakResult<Self> where P : AsRef<Path> {
        let file = File::open(path)?;
        Self::new(BufReader::new(file))
    }
    
    /// Loads an object from the pak file via queried indices. This will only load the necessary data into memory.
    pub fn query<T>(&self, query : impl PakQueryExpression<T>) -> PakResult<T::ReturnType> where T : DeserializeGroup  {
        let pointers = query.execute(self)?.into_iter().collect::<OrderSet<_>>();
        T::deserialize_group(self, pointers)
    }
    
    /// Loads an object from the pak file via queried indices. This will only load the necessary data into memory.
    pub fn query_pql<T>(&self, pql : &str) -> PakResult<T::ReturnType> where T : DeserializeGroup + 'static  {
        let query = crate::query::pql::pql(pql)?;
        self.query::<T>(query)
    }
    
    /// Returns the size of the pak file in bytes.
    pub fn size(&self) -> u64 {
        24 + self.inner.sizing.meta_size + self.inner.sizing.indices_size + self.inner.sizing.vault_size
    }
    
    /// Returns the name given to the pak file.
    pub fn name(&self) -> &str {
        &self.inner.meta.name
    }
    
    /// Returns the version of the pak file.
    pub fn version(&self) -> &str {
        &self.inner.meta.version
    }
    
    /// Returns the author of the pak file.
    pub fn author(&self) -> &str {
        &self.inner.meta.author
    }
    
    /// Returns the description of the pak file.
    pub fn description(&self) -> &str {
        &self.inner.meta.description
    }
    
    /// This returns the extra data that can be saved in the metadata. This can throw an error if the
    /// wrong type is asked for.
    pub fn get_extra<T>(&self) -> PakResult<T> where T : for<'de> Deserialize<'de> {
        self.inner.meta.get_extra()
    }
    
    /// Read Data directly with a pointer. 
    pub fn read<T>(&self, pointer : &PakPointer) -> PakResult<T> where T : PakDeserialize {
        self.inner.read(pointer)
    }
    
    /// Read Data directly with a pointer. 
    pub fn read_serde<T>(&self, pointer : &PakPointer) -> PakResult<T> where T : for<'de> Deserialize<'de> {
        self.inner.read_serde(pointer)
    }
    
    pub fn identifier(&self) -> &str {
        &self.inner.meta.identifier
    }
    
    pub(crate) fn weak(&self) -> Weak<PakInner> {
        Arc::downgrade(&self.inner)
    }
    
    pub fn check_identifier(&self, id : &str) -> PakResult<()> {
        if self.identifier() != id { return Err(error::PakError::PakIdentifierMismatch)}
        Ok(())
    }
    
    pub(crate) fn get_tree(&self, key : &str) -> PakResult<PakTree<'_>> {
        PakTree::new(self, key)
    }
    
    pub(crate) fn fetch_indices(&self) -> PakResult<HashMap<String, PakUntypedPointer>> {
        let pointer = PakPointer::new_untyped(self.get_indices_start(), self.inner.sizing.indices_size);
        let Ok(mut source) = self.inner.source.write() else { return Err(error::PakError::SourceInUse) };
        let buffer = source.read(&pointer, 0)?;
        let indices = bincode::deserialize(&buffer)?;
        Ok(indices)
    }
    
    pub(crate) fn fetch_all_pointers_of<T>(&self) -> PakResult<Vec<PakPointer>> where T : DeserializeGroup {
        let Ok(mut source) = self.inner.source.write() else { return Err(error::PakError::SourceInUse) };
        let lists_pointer = PakPointer::new_untyped(self.get_list_start(), self.inner.sizing.list_size);
        let lists_buffer = source.read(&lists_pointer, 0)?;
        let lists : HashMap<String, PakPointer> = bincode::deserialize(&lists_buffer)?;
        
        let mut values = Vec::new();
        drop(source);
        for type_name in T::get_types() {
            let Some(list_pointer) = lists.get(type_name) else { continue };
            let list = self.inner.read_serde::<Vec<PakPointer>>(list_pointer)?;
            list.iter()
                .map(|pointer| PakTypedPointer::new(pointer.offset(), pointer.size(), type_name).into_pointer())
                .for_each(|pointer| values.push(pointer));
        }
        
        Ok(values)
    }
    
    pub(crate) fn get_list_start(&self) -> u64 {
        self.inner.get_list_start()
    }
    
    pub(crate) fn get_indices_start(&self) -> u64 {
        self.inner.get_indices_start()
    }
}

//==============================================================================================
//        PakInner
//==============================================================================================

pub struct PakInner {
    sizing : PakSizing,
    meta : PakMeta,
    source : RwLock<Box<dyn PakSource + Send + Sync + 'static>>,
}

impl PakInner {
    pub fn read<T>(&self, pointer : &PakPointer) -> PakResult<T> where T : PakDeserialize {
        if !pointer.type_is_match::<T>() { return Err(error::PakError::TypeMismatchError(pointer.type_name().to_string(), std::any::type_name::<T>().to_string())) }
        T::unpak(self, pointer)
    }
    
    pub fn read_serde<T>(&self, pointer : &PakPointer) -> PakResult<T> where T : for<'de> Deserialize<'de> {
        if !pointer.type_is_match::<T>() { return Err(error::PakError::TypeMismatchError(pointer.type_name().to_string(), std::any::type_name::<T>().to_string())) }
        let buffer = self.read_raw(pointer)?;
        Ok(bincode::deserialize(buffer.as_slice())?)
    }
    
    pub fn read_raw(&self, pointer : &PakPointer) -> PakResult<Vec<u8>> {
        let Ok(mut source) = self.source.write() else { return Err(error::PakError::SourceInUse) };
        source.read(pointer, self.get_vault_start())
    }
    
    fn get_vault_start(&self) -> u64 {
        // To be honest, I'm not sure why this start is offset by 8, it just is and I am to scared to ask.
        self.get_list_start() + self.sizing.list_size + 8
    }
    
    fn get_list_start(&self) -> u64 {
        self.get_indices_start() + self.sizing.indices_size
    }
    
    fn get_indices_start(&self) -> u64 {
        PAK_SIZING_STRUCT_SIZE_IN_BYTES + self.sizing.meta_size
    }
}

//==============================================================================================
//        PakSource
//==============================================================================================

///This is where a Pak file will load from. This trait is automatically implemented for any type that implements [Read](std::io::Read) and [Seek](std::io::Seek).
pub trait PakSource {
    ///Returns data from the source based on a [PakPointer](crate::PakPointer)
    fn read(&mut self, pointer : &PakPointer, offset : u64) -> PakResult<Vec<u8>>;
}

impl <R> PakSource for R where R : Read + Seek {
    fn read(&mut self, pointer : &PakPointer, offset : u64) -> PakResult<Vec<u8>> {
        let mut buffer = vec![0u8; pointer.size() as usize];
        self.seek(SeekFrom::Start(pointer.offset() + offset))?;
        self.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}

