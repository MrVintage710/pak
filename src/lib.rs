#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/MrVintage710/pak/refs/heads/main/docs/icon.png")]

use std::{cell::RefCell, collections::HashMap, fmt::Debug, fs::{self, File}, io::{BufReader, Cursor, Read, Seek, SeekFrom}, path::Path};
use btree::{PakTree, PakTreeBuilder};
use index::PakIndex;
use item::{PakItemDeserialize, PakItemDeserializeGroup, PakItemSearchable, PakItemSerialize};
use meta::{PakMeta, PakSizing};
use pointer::{PakPointer, PakTypedPointer, PakUntypedPointer};
use query::PakQueryExpression;

use crate::error::PakResult;

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

//==============================================================================================
//        Pak File
//==============================================================================================

/// Represents a Pak file. This struct provides access to the metadata and data stored within the Pak file.
pub struct Pak {
    sizing : PakSizing,
    meta : PakMeta,
    source : RefCell<Box<dyn PakSource>>
}

impl Pak {
    /// Creates a new Pak instance from a [PakSource](crate::PakSource).
    pub fn new<S>(mut source : S) -> PakResult<Self> where S : PakSource + 'static {
        let sizing_pointer = PakPointer::new_untyped(0, 24);
        let sizing_buffer = source.read(&sizing_pointer, 0)?;
        let sizing : PakSizing = bincode::deserialize(&sizing_buffer)?;
        
        let meta_pointer = PakPointer::new_untyped(24, sizing.meta_size);
        let meta_buffer = source.read(&meta_pointer, 0)?;
        let meta : PakMeta = bincode::deserialize(&meta_buffer)?;

        Ok(Self { sizing, source : RefCell::new(Box::new(source)), meta })
    }
    
    /// Loads a Pak from the specified file path. This will not load the entire pak file into memory, just the header.
    pub fn new_from_file<P>(path : P) -> PakResult<Self> where P : AsRef<Path> {
        let file = File::open(path)?;
        Self::new(BufReader::new(file))
    }
    
    /// Loads an object from the pak file via queried indices. This will only load the necessary data into memory.
    pub fn query<T>(&self, query : impl PakQueryExpression) -> PakResult<T::ReturnType> where T : PakItemDeserializeGroup  {
        let pointers = query.execute(self)?.into_iter().map(|i| i.into_pointer()).collect();
        T::deserialize_group(self, pointers)
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
    
    pub(crate) fn read_err<T>(&self, pointer : &PakPointer) -> PakResult<T> where T : PakItemDeserialize {
        if !pointer.type_is_match::<T>() { return Err(error::PakError::TypeMismatchError(pointer.type_name().to_string(), std::any::type_name::<T>().to_string())) }
        let buffer = self.source.borrow_mut().read(pointer, self.get_vault_start())?;
        let res = T::from_bytes(&buffer)?;
        Ok(res)
    }
    
    pub(crate) fn read<T>(&self, pointer : &PakPointer) -> Option<T> where T : PakItemDeserialize {
        let res = self.read_err(pointer);
        match res {
            Ok(res) => Some(res),
            Err(_) => None,
        }
    }
    
    pub(crate) fn get_tree(&self, key : &str) -> PakResult<PakTree> {
        PakTree::new(self, key)
    }
    
    pub(crate) fn fetch_indices(&self) -> PakResult<HashMap<String, PakUntypedPointer>> {
        let pointer = PakPointer::new_untyped(self.get_indices_start(), self.sizing.indices_size);
        let buffer = self.source.borrow_mut().read(&pointer, 0)?;
        let indices = bincode::deserialize(&buffer)?;
        Ok(indices)
    }
    
    pub(crate) fn get_vault_start(&self) -> u64 {
        // To be honest, I'm not sure why this start is offset by 8, it just is and I am to scared to ask.
        24 + self.sizing.meta_size + self.sizing.indices_size + 8
    }
    
    pub(crate) fn get_indices_start(&self) -> u64 {
        24 + self.sizing.meta_size
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

//==============================================================================================
//        PakBuilder
//==============================================================================================

/// When it is time to create the pak file, this struct is used to build it. Remember that this struct doen't have the ability to read data that has been paked or delete data that has been paked.
pub struct PakBuilder {
    chunks : Vec<PakVaultReference>,
    size_in_bytes : u64,
    vault : Vec<u8>,
    name: String,
    description: String,
    author: String,
}

impl PakBuilder {
    /// Creates a new instance of PakBuilder.
    pub fn new() -> Self {
        Self {
            vault : Vec::new(),
            chunks : Vec::new(),
            size_in_bytes : 0,
            name: String::new(),
            description: String::new(),
            author: String::new(),
        }
    }
    
    /// Adds an item to the pak file that does not support searching. Takes anything that implements [PakItemSerialize](crate::PakItemSerialize).
    pub fn pak_no_search<T: PakItemSerialize>(&mut self, item : T) -> PakResult<PakPointer> {
        let bytes = item.into_bytes()?;
        let pointer = PakPointer::new_typed::<T>(self.size_in_bytes, bytes.len() as u64);
        self.size_in_bytes += bytes.len() as u64;
        self.vault.extend(bytes);
        self.chunks.push(PakVaultReference { pointer: pointer.clone().into_typed::<T>(), indices: vec![] });
        Ok(pointer)
    }
    
    /// Adds an item to the pak file that supports searching. Takes anything that implements [PakItemSerialize](crate::PakItemSerialize) and [PakItemSearchable](crate::PakItemSearchable).
    pub fn pak<T : PakItemSerialize + PakItemSearchable>(&mut self, item : T) -> PakResult<PakPointer> {
        let indices = item.get_indices();
        let bytes = item.into_bytes()?;
        let pointer = PakPointer::new_typed::<T>(self.size_in_bytes, bytes.len() as u64);
        self.size_in_bytes += bytes.len() as u64;
        self.vault.extend(bytes);
        self.chunks.push(PakVaultReference { pointer: pointer.clone().into_typed::<T>(), indices: indices.clone() });
        Ok(pointer)
    }
    
    /// The current size of the pak file in bytes.
    pub fn size(&self) -> u64 {
        self.size_in_bytes
    }
    
    /// The number of items in the pak file.
    pub fn len(&self) -> usize {
        self.chunks.len()
    }
    
    /// Adds a name to the pak file's metadata.
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
    
    /// Adds a description to the pak file's metadata.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }
    
    /// Adds an author to the pak file's metadata.
    pub fn with_author(mut self, author: &str) -> Self {
        self.author = author.to_string();
        self
    }
    
    /// Sets the name of the pak file's metadata.
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
    
    /// Sets the description of the pak file's metadata.
    pub fn set_description(&mut self, description: &str) {
        self.description = description.to_string();
    }
    
    /// Sets the author of the pak file's metadata.
    pub fn set_author(&mut self, author: &str) {
        self.author = author.to_string();
    }
    
    /// Builds the pak file and writes it to the specified path. This also returns a [Pak](crate::Pak) object that is attached to that file.
    pub fn build_file(self, path : impl AsRef<Path>) -> PakResult<Pak> {
        let (out, sizing, meta) = self.build_internal()?;
        
        fs::write(&path, out)?;
        let pak  = Pak {
            sizing,
            meta,
            source: RefCell::new(Box::new(BufReader::new(File::open(path)?))),
        };
        Ok(pak)
    }
    
    /// Builds the pak file and writes it to the specified path. This also returns a [Pak](crate::Pak) object that is attached to that slice of memory.
    pub fn build_in_memory(self) -> PakResult<Pak> {
        let (out, sizing, meta) = self.build_internal()?;
        
        let pak = Pak {
            sizing,
            meta,
            source: RefCell::new(Box::new(Cursor::new(out))),
        };
        Ok(pak)
    }
    
    fn build_internal(mut self)  -> PakResult<(Vec<u8>, PakSizing, PakMeta)> {
        let mut map : HashMap<String, PakTreeBuilder> = HashMap::new();
        for chunk in &self.chunks {
            for index in &chunk.indices{
                map.entry(index.key.clone())
                    .or_insert(PakTreeBuilder::new(6))
                    .access()
                    .insert(index.value.clone(), chunk.pointer.clone())
                ;
            }
        }
        
        let mut pointer_map : HashMap<String, PakUntypedPointer> = HashMap::new();
        for (key, tree) in map {
            let pointer = tree.into_pak(&mut self)?;
            pointer_map.insert(key, pointer.as_untyped());
        }
        
        let meta = PakMeta {
            name: self.name,
            description: self.description,
            author: self.author,
            version: "1.0".to_string(),
        };
        
        let sizing = PakSizing {
            meta_size: bincode::serialized_size(&meta)?,
            indices_size: bincode::serialized_size(&pointer_map)?,
            vault_size: bincode::serialized_size(&self.vault)?,
        };
        
        let mut sizing_out = bincode::serialize(&sizing)?;
        let mut meta_out = bincode::serialize(&meta)?;
        let mut pointer_map_out = bincode::serialize(&pointer_map)?;
        let mut vault_out = bincode::serialize(&self.vault)?;
        
        let mut out = Vec::<u8>::new();
        out.append(&mut sizing_out);
        out.append(&mut meta_out);
        out.append(&mut pointer_map_out);
        out.append(&mut vault_out);
        Ok((out, sizing, meta))
    }
    
}

//==============================================================================================
//        PakVaultReference
//==============================================================================================

#[derive(Debug, Clone)]
pub(crate) struct PakVaultReference {
    pointer : PakTypedPointer,
    indices : Vec<PakIndex>
}