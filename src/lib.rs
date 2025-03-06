use std::{cell::RefCell, collections::HashMap, fmt::Debug, fs::{self, File}, io::{BufReader, Cursor, Read, Seek, SeekFrom}, path::Path};

use bincode::Options;
use btree::{PakTree, PakTreeBuilder};
use index::PakIndex;
use item::{PakItemDeserialize, PakItemDeserializeGroup, PakItemSearchable, PakItemSerialize};
use meta::{PakMeta, PakSizing};
use query::PakQueryExpression;
use serde::{Deserialize, Serialize};
use value::PakValue;

use crate::error::PakResult;

pub mod meta;
pub mod item;
pub mod index;
pub mod value;
pub mod btree;
pub mod query;
pub mod error;

#[cfg(test)]
mod test;

//==============================================================================================
//        Pak File
//==============================================================================================

pub struct Pak {
    sizing : PakSizing,
    meta : PakMeta,
    source : RefCell<Box<dyn PakSource>>
}

// impl Pak<BufReader<File>> {
//     pub fn open(path : impl AsRef<Path>) -> PakResult<Self> {
//         let file = File::open(path)?;
//         let mut stream = BufReader::new(file);
        
//         let mut sizing_buffer = [0u8; 24];
//         stream.read_exact(&mut sizing_buffer)?;
//         let sizing : PakSizing = bincode::deserialize(&sizing_buffer)?;

//         let mut meta_buffer = vec![0u8; sizing.meta_size as usize];
//         stream.seek(SeekFrom::Start(24))?;
//         stream.read_exact(&mut meta_buffer)?;
//         let meta : PakMeta = bincode::deserialize(&meta_buffer)?;

//         Ok(Self { sizing, source : RefCell::new(stream), meta })
//     }
// }

// impl Pak<Cursor<Vec<u8>>> {
//     pub fn open_in_memory(data : Vec<u8>) -> PakResult<Self> {
//         let mut stream = Cursor::new(data);
        
//         let mut sizing_buffer = [0u8; 24];
//         stream.read_exact(&mut sizing_buffer)?;
//         let sizing : PakSizing = bincode::deserialize(&sizing_buffer)?;

//         let mut meta_buffer = vec![0u8; sizing.meta_size as usize];
//         stream.seek(SeekFrom::Start(24))?;
//         stream.read_exact(&mut meta_buffer)?;
//         let meta : PakMeta = bincode::deserialize(&meta_buffer)?;

//         Ok(Self { sizing, source : RefCell::new(stream), meta })
//     }
// }

impl Pak {
    pub fn new<S>(mut source : S) -> PakResult<Self> where S : PakSource + 'static {
        let sizing_pointer = PakPointer::new(0, 24);
        let sizing_buffer = source.read(sizing_pointer, 0)?;
        let sizing : PakSizing = bincode::deserialize(&sizing_buffer)?;
        
        let meta_pointer = PakPointer::new(24, sizing.meta_size);
        let meta_buffer = source.read(meta_pointer, 0)?;
        let meta : PakMeta = bincode::deserialize(&meta_buffer)?;

        Ok(Self { sizing, source : RefCell::new(Box::new(source)), meta })
    }
    
    pub(crate) fn read_err<T>(&self, pointer : PakPointer) -> PakResult<T> where T : PakItemDeserialize {
        let buffer = self.source.borrow_mut().read(pointer, self.get_vault_start())?;
        let res = T::from_bytes(&buffer)?;
        println!("Reading {} from {pointer:?}:   Buffer: {buffer:?}", std::any::type_name::<T>());
        Ok(res)
    }
    
    pub(crate) fn read<T>(&self, pointer : PakPointer) -> Option<T> where T : PakItemDeserialize {
        let res = self.read_err(pointer);
        match res {
            Ok(res) => Some(res),
            Err(_) => None,
        }
    }
    
    pub(crate) fn get_tree(&self, key : &str) -> PakResult<PakTree> {
        PakTree::new(self, key)
    }
    
    pub fn fetch_indices(&self) -> PakResult<HashMap<String, PakPointer>> {
        let pointer = PakPointer::new(self.get_indices_start(), self.sizing.indices_size);
        let buffer = self.source.borrow_mut().read(pointer, 0)?;
        let indices = bincode::deserialize(&buffer)?;
        Ok(indices)
    }
    
    pub fn query<T>(&self, query : impl PakQueryExpression) -> PakResult<T::ReturnType> where T : PakItemDeserializeGroup  {
        let pointers = query.execute(self)?;
        // let values = pointers.into_iter().filter_map(|pointer| self.read::<T>(pointer)).collect::<Vec<_>>();
        T::deserialize_group(self, pointers)
    }
    
    pub fn get_vault_start(&self) -> u64 {
        // To be honest, I'm not sure why this start is offset by 8, it just is and I am to scared to ask.
        24 + self.sizing.meta_size + self.sizing.indices_size + 8
    }
    
    pub fn get_indices_start(&self) -> u64 {
        24 + self.sizing.meta_size
    }
    
    pub fn size(&self) -> u64 {
        24 + self.sizing.meta_size + self.sizing.indices_size + self.sizing.vault_size
    }
}

//==============================================================================================
//        PakPointer
//==============================================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize, Hash)]
pub struct PakPointer {
    offset : u64,
    size : u64,
}

impl PakPointer {
    pub fn new(offset : u64, size : u64) -> Self {
        Self { offset, size }
    }
}

//==============================================================================================
//        PakSource
//==============================================================================================

pub trait PakSource {
    fn read(&mut self, pointer : PakPointer, offest : u64) -> PakResult<Vec<u8>>;
}

impl <R> PakSource for R where R : Read + Seek {
    fn read(&mut self, pointer : PakPointer, offest : u64) -> PakResult<Vec<u8>> {
        let mut buffer = vec![0u8; pointer.size as usize];
        self.seek(SeekFrom::Start(pointer.offset + offest))?;
        self.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}

//==============================================================================================
//        PakBuilder
//==============================================================================================

pub struct PakBuilder {
    chunks : Vec<PakVaultReference>,
    size_in_bytes : u64,
    vault : Vec<u8>,
    name: String,
    description: String,
    author: String,
}

impl PakBuilder {
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
    
    pub fn pak_no_search<T: PakItemSerialize>(&mut self, item : T) -> PakResult<PakPointer> {
        let bytes = item.into_bytes()?;
        let pointer = PakPointer::new(self.size_in_bytes, bytes.len() as u64);
        self.size_in_bytes += bytes.len() as u64;
        self.vault.extend(bytes);
        self.chunks.push(PakVaultReference { pointer, indices: vec![] });
        Ok(pointer)
    }
    
    pub fn pak<T : PakItemSerialize + PakItemSearchable>(&mut self, item : T) -> PakResult<PakVaultReference> {
        let indices = item.get_indices();
        let bytes = item.into_bytes()?;
        let pointer = PakPointer::new(self.size_in_bytes, bytes.len() as u64);
        self.size_in_bytes += bytes.len() as u64;
        self.vault.extend(bytes);
        self.chunks.push(PakVaultReference { pointer, indices: indices.clone() });
        Ok(PakVaultReference { pointer, indices })
    }
    
    pub fn size(&self) -> u64 {
        self.size_in_bytes
    }
    
    pub fn len(&self) -> usize {
        self.chunks.len()
    }
    
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
    
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }
    
    pub fn with_author(mut self, author: &str) -> Self {
        self.author = author.to_string();
        self
    }
    
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
    
    pub fn set_description(&mut self, description: &str) {
        self.description = description.to_string();
    }
    
    pub fn set_author(&mut self, author: &str) {
        self.author = author.to_string();
    }
    
    fn build_internal(mut self)  -> PakResult<(Vec<u8>, PakSizing, PakMeta)> {
        let mut map : HashMap<String, PakTreeBuilder> = HashMap::new();
        for chunk in &self.chunks {
            for index in &chunk.indices{
                map.entry(index.key.clone())
                    .or_insert(PakTreeBuilder::new(6))
                    .access()
                    .insert(index.value.clone(), chunk.pointer)
                ;
            }
        }
        
        let mut pointer_map : HashMap<String, PakPointer> = HashMap::new();
        for (key, tree) in map {
            let pointer = tree.into_pak(&mut self)?;
            pointer_map.insert(key, pointer);
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
    
    pub fn build_in_memory(self) -> PakResult<Pak> {
        let (out, sizing, meta) = self.build_internal()?;
        
        let pak = Pak {
            sizing,
            meta,
            source: RefCell::new(Box::new(Cursor::new(out))),
        };
        Ok(pak)
    }
}

//==============================================================================================
//        PakVaultReference
//==============================================================================================

#[derive(Debug)]
pub struct PakVaultReference {
    pub pointer : PakPointer,
    pub indices : Vec<PakIndex>
}