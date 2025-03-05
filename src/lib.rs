use std::{cell::RefCell, collections::HashMap, fs::{self, File}, io::{BufReader, Read, Seek, SeekFrom}, path::Path};

use btree::{PakTree, PakTreeBuilder};
use index::{PakIndex, PakIndices};
use item::{PakItemDef, PakItemRef, PakItemSearchable};
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

//==============================================================================================
//        Pak File
//==============================================================================================

pub struct Pak {
    sizing : PakSizing,
    source : RefCell<BufReader<File>>
}

impl Pak {
    pub fn open(path : impl AsRef<Path>) -> PakResult<Self> {
        let file = File::open(path)?;
        let mut stream = BufReader::new(file);
        
        let mut sizing_buffer = [0u8; 24];
        stream.read_exact(&mut sizing_buffer)?;
        let sizing : PakSizing = bincode::deserialize(&sizing_buffer)?;

        let mut meta_buffer = vec![0u8; sizing.meta_size as usize];
        stream.seek(SeekFrom::Start(24))?;
        stream.read_exact(&mut meta_buffer)?;
        let meta : PakMeta = bincode::deserialize(&meta_buffer)?;

        Ok(Self { sizing, source : RefCell::new(stream) })
    }
    
    pub(crate) fn read_err<T>(&self, pointer : PakPointer) -> PakResult<T> where T : PakItemRef {
        let mut stream = self.source.borrow_mut();
        let mut buffer = vec![0u8; pointer.size as usize];
        stream.seek(SeekFrom::Start(pointer.offset + self.get_vault_start()))?;
        stream.read_exact(&mut buffer)?;
        let res = T::from_bytes(&buffer)?;
        Ok(res)
    }
    
    pub(crate) fn read<T>(&self, pointer : PakPointer) -> Option<T> where T : PakItemRef {
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
        let mut stream = self.source.borrow_mut();
        let mut buffer = vec![0u8; self.sizing.indices_size as usize];
        stream.seek(SeekFrom::Start(self.get_indices_start()))?;
        stream.read_exact(&mut buffer)?;
        let indices = bincode::deserialize(&buffer)?;
        Ok(indices)
    }
    
    pub fn query<T>(&self, query : impl PakQueryExpression) -> PakResult<Vec<T>> where T : PakItemRef  {
        let pointers = query.execute(self)?;
        let values = pointers.into_iter().filter_map(|pointer| self.read::<T>(pointer)).collect::<Vec<_>>();
        Ok(values)
    }
    
    pub fn search<T>(&self, key : &str, value : impl Into<PakValue>) -> PakResult<Vec<T>> where T : PakItemRef {
        let value = value.into();
        let index_types = self.fetch_indices()?;
        let Some(pointer) = index_types.get(key) else {return Ok(vec![])};
        let indices : PakIndices = self.read_err(*pointer)?;
        let Some(object_pointers) = indices.get(&value) else {return Ok(vec![])};
        let res = object_pointers.iter().filter_map(|pointer| self.read::<T>(*pointer)).collect::<Vec<_>>();
        
        Ok(res)
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
//        PakVault
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
    
    pub fn pak_no_search<T: PakItemDef>(&mut self, item : T) -> PakResult<PakPointer> {
        let bytes = item.into_bytes()?;
        let pointer = PakPointer::new(self.size_in_bytes, bytes.len() as u64);
        self.size_in_bytes += bytes.len() as u64;
        self.vault.extend(bytes);
        self.chunks.push(PakVaultReference { pointer, indices: vec![] });
        Ok(pointer)
    }
    
    pub fn pak<T : PakItemDef + PakItemSearchable>(&mut self, item : T) -> PakResult<PakVaultReference> {
        let indices = item.get_indices();
        let bytes = item.into_bytes()?;
        let pointer = PakPointer::new(self.size_in_bytes, bytes.len() as u64);
        self.size_in_bytes += bytes.len() as u64;
        self.vault.extend(bytes);
        self.chunks.push(PakVaultReference { pointer, indices: indices.clone() });
        Ok(PakVaultReference { pointer, indices })
    }
    
    pub fn unpak<T>(&self, pointer : &PakPointer) -> PakResult<T> where T : PakItemRef {
        let res = T::from_pak(&self.vault, *pointer)?;
        Ok(res)
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
    
    pub fn build_in_memory(mut self)  -> PakResult<(Vec<u8>, PakSizing, PakMeta)> {
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
    
    pub fn build(self, path : impl AsRef<Path>) -> PakResult<Pak> {
        
        let (out, sizing, meta) = self.build_in_memory()?;
        
        fs::write(&path, out)?;
        let pak = Pak {
            sizing,
            source: RefCell::new(BufReader::new(File::open(path)?)),
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

//==============================================================================================
//        Tests
//==============================================================================================

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use super::*;
    
    static INIT: Once = Once::new();
    
    pub fn initialize() {
        INIT.call_once(|| {
            let mut builder = PakBuilder::new();
            
            let person1 = Person {
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
            };
            
            let person2 = Person {
                first_name: "Jane".to_string(),
                last_name: "Doe".to_string(),
            };
            
            let person3 = Person {
                first_name: "Alice".to_string(),
                last_name: "Smith".to_string(),
            };
            
            builder.pak(person1).unwrap();
            builder.pak(person2).unwrap();
            builder.pak(person3).unwrap();
            
            builder.build("test.pak").unwrap();
        });
    }
    
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    struct Person {
        first_name: String,
        last_name: String,
    }
    
    impl PakItemSearchable for Person {
        fn get_indices(&self) -> Vec<PakIndex> {
            let mut indices = Vec::new();
            indices.push(PakIndex::new("first_name", self.first_name.clone()));
            indices.push(PakIndex::new("last_name", self.last_name.clone()));
            Vec::new()
        }
    }
    
    #[test]
    fn test_pak_fetch_indices() { 
        initialize();   
        let pak = Pak::open("test.pak").unwrap();
        let indices = pak.fetch_indices().unwrap();
        assert_eq!(indices.len(), 2);
    }
    
    #[test]
    fn test_pak_read() { 
        initialize();
        let pak = Pak::open("test.pak").unwrap();
        let person : Person = pak.read(PakPointer::new(0, 23)).unwrap();
        assert_eq!(person.first_name, "John".to_string());
    }
    
    #[test]
    fn test_pak_search() {
        initialize(); 
        let pak = Pak::open("test.pak").unwrap();
        let res : Vec<Person> = pak.search("last_name", "Doe").unwrap();
        assert_eq!(res.len(), 2);
    }
}