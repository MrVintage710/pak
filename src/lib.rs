#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/MrVintage710/pak/refs/heads/main/docs/icon.png")]

use std::{collections::HashMap, fs::File, io::{BufReader, Read, Seek, SeekFrom}, path::Path, sync::{RwLock, RwLockWriteGuard}};
use btree::PakTree;
use item::{PakItemDeserialize, PakItemDeserializeGroup};
use meta::{PakMeta, PakSizing};
use pointer::{PakPointer, PakUntypedPointer};
use query::PakQueryExpression;

use crate::{error::PakResult};

#[cfg(test)]
mod test;

pub mod meta;
pub mod item;
pub mod index;
pub mod value;
pub(crate) mod btree;
pub mod query;
pub mod error;
pub mod pointer;
pub mod builder;

//==============================================================================================
//        Pak File
//==============================================================================================

pub const PAK_FILE_VERSION : &'static str = "1.1";

/// Represents a Pak file. This struct provides access to the metadata and data stored within the Pak file.
pub struct Pak {
    sizing : PakSizing,
    meta : PakMeta,
    source : RwLock<Box<dyn PakSource + Send + Sync + 'static>>
}

impl Pak {
    /// Creates a new Pak instance from a [PakSource](crate::PakSource).
    pub fn new<S>(mut source : S) -> PakResult<Self> where S : PakSource + Send + Sync + 'static {
        let sizing_pointer = PakPointer::new_untyped(0, 24);
        let sizing_buffer = source.read(&sizing_pointer, 0)?;
        let sizing : PakSizing = bincode::deserialize(&sizing_buffer)?;
        
        let meta_pointer = PakPointer::new_untyped(24, sizing.meta_size);
        let meta_buffer = source.read(&meta_pointer, 0)?;
        let meta : PakMeta = bincode::deserialize(&meta_buffer)?;

        Ok(Self { sizing, source : RwLock::new(Box::new(source)), meta })
    }
    
    /// Loads a Pak from the specified file path. This will not load the entire pak file into memory, just the header.
    pub fn new_from_file<P>(path : P) -> PakResult<Self> where P : AsRef<Path> {
        let file = File::open(path)?;
        Self::new(BufReader::new(file))
    }
    
    /// Loads an object from the pak file via queried indices. This will only load the necessary data into memory.
    pub fn query<T>(&self, query : impl PakQueryExpression<T>) -> PakResult<T::ReturnType> where T : PakItemDeserializeGroup  {
        let pointers = query.execute(self)?.into_iter().collect();
        T::deserialize_group(self, pointers)
    }
    
    /// Loads an object from the pak file via queried indices. This will only load the necessary data into memory.
    pub fn query_sql<T>(&self, pql : &str) -> PakResult<T::ReturnType> where T : PakItemDeserializeGroup + 'static  {
        let query = crate::query::pql::pql(pql)?;
        self.query::<T>(query)
    }
    
    /// Returns the size of the pak file in bytes.
    pub fn size(&self) -> u64 {
        24 + self.sizing.meta_size + self.sizing.indices_size + self.sizing.vault_size
    }
    
    /// Returns the name given to the pak file.
    pub fn name(&self) -> &str {
        &self.meta.name
    }
    
    /// Returns the version of the pak file.
    pub fn version(&self) -> &str {
        &self.meta.version
    }
    
    /// Returns the author of the pak file.
    pub fn author(&self) -> &str {
        &self.meta.author
    }
    
    /// Returns the description of the pak file.
    pub fn description(&self) -> &str {
        &self.meta.description
    }
    
    pub fn read_err<T>(&self, pointer : &PakPointer) -> PakResult<T> where T : PakItemDeserialize {
        if !pointer.type_is_match::<T>() { return Err(error::PakError::TypeMismatchError(pointer.type_name().to_string(), std::any::type_name::<T>().to_string())) }
        let Ok(mut source) = self.source.write() else { return Err(error::PakError::SourceInUse)};
        self.read_internal(pointer, &mut source)
    }
    
    pub fn read<T>(&self, pointer : &PakPointer) -> Option<T> where T : PakItemDeserialize {
        self.read_err::<T>(pointer).ok()
    }
    
    fn read_internal<T>(&self, pointer : &PakPointer, source : &mut RwLockWriteGuard<Box<dyn PakSource + Send + Sync + 'static>>) -> PakResult<T> where T : PakItemDeserialize {
        let buffer = source.read(pointer, self.get_vault_start())?;
        let res = T::from_bytes(&buffer)?;
        Ok(res)
    }
    
    pub(crate) fn get_tree(&self, key : &str) -> PakResult<PakTree<'_>> {
        PakTree::new(self, key)
    }
    
    pub(crate) fn fetch_indices(&self) -> PakResult<HashMap<String, PakUntypedPointer>> {
        let pointer = PakPointer::new_untyped(self.get_indices_start(), self.sizing.indices_size);
        let Ok(mut source) = self.source.write() else { return Err(error::PakError::SourceInUse) };
        let buffer = source.read(&pointer, 0)?;
        let indices = bincode::deserialize(&buffer)?;
        Ok(indices)
    }
    
    pub(crate) fn fetch_all_pointers_of<T>(&self) -> PakResult<Vec<PakPointer>> where T : PakItemDeserializeGroup {
        let Ok(mut source) = self.source.write() else { return Err(error::PakError::SourceInUse) };
        let lists_pointer = PakPointer::new_untyped(self.get_list_start(), self.sizing.list_size);
        let lists_buffer = source.read(&lists_pointer, 0)?;
        let lists : HashMap<String, PakPointer> = bincode::deserialize(&lists_buffer)?;
        let values = T::get_types().into_iter()
            .filter_map(|type_name| lists.get(type_name))
            .filter_map(|pointer| self.read_internal::<Vec<PakPointer>>(pointer, &mut source).ok())
            .flatten()
            .collect::<Vec<_>>();
        Ok(values)
    }
    
    pub(crate) fn get_vault_start(&self) -> u64 {
        // To be honest, I'm not sure why this start is offset by 8, it just is and I am to scared to ask.
        self.get_list_start() + self.sizing.list_size + 8
    }
    
    pub(crate) fn get_list_start(&self) -> u64 {
        self.get_indices_start() + self.sizing.indices_size
    }
    
    pub(crate) fn get_indices_start(&self) -> u64 {
        32 + self.sizing.meta_size
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

