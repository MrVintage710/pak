use std::{collections::HashMap, fmt::Debug, fs::{self, File}, io::{BufReader, Cursor}, path::Path, sync::RwLock};


use crate::{PAK_FILE_VERSION, Pak, btree::PakTreeBuilder, error::PakResult, index::PakIndex, item::{PakItemDeserialize, PakItemSearchable, PakItemSerialize}, meta::{PakMeta, PakSizing}, pointer::{PakPointer, PakUntypedPointer}};

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
    pub fn pak_no_search<T: PakItemSerialize + Debug + PakItemDeserialize>(&mut self, item : T) -> PakResult<PakPointer> {
        let bytes = item.into_bytes()?;
        let pointer = PakPointer::new_untyped(self.size_in_bytes, bytes.len() as u64);
        self.size_in_bytes += bytes.len() as u64;
        self.vault.extend(bytes);
        self.chunks.push(PakVaultReference { pointer: pointer.clone(), indices: vec![] });
        Ok(pointer)
    }
    
    /// Adds an item to the pak file that supports searching. Takes anything that implements [PakItemSerialize](crate::PakItemSerialize) and [PakItemSearchable](crate::PakItemSearchable).
    pub fn pak<T : PakItemSerialize + PakItemSearchable>(&mut self, item : T) -> PakResult<PakPointer> {
        let indices = item.get_indices();
        let bytes = item.into_bytes()?;
        let pointer = PakPointer::new_typed::<T>(self.size_in_bytes, bytes.len() as u64);
        self.size_in_bytes += bytes.len() as u64;
        self.vault.extend(bytes);
        self.chunks.push(PakVaultReference { pointer: pointer.clone(), indices: indices.clone() });
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
            source: RwLock::new(Box::new(BufReader::new(File::open(path)?))),
        };
        Ok(pak)
    }
    
    /// Builds the pak file and writes it to the specified path. This also returns a [Pak](crate::Pak) object that is attached to that slice of memory.
    pub fn build_in_memory(self) -> PakResult<Pak> {
        let (out, sizing, meta) = self.build_internal()?;
        
        let pak = Pak {
            sizing,
            meta,
            source: RwLock::new(Box::new(Cursor::new(out))),
        };
        Ok(pak)
    }
    
    fn build_internal(mut self)  -> PakResult<(Vec<u8>, PakSizing, PakMeta)> {
        let mut map : HashMap<String, PakTreeBuilder> = HashMap::new();
        let mut list_values : HashMap<String, Vec<PakPointer>> = HashMap::new();
        for chunk in &self.chunks {
            for index in &chunk.indices{
                map.entry(index.key.clone())
                    .or_insert(PakTreeBuilder::new(16))
                    .access()
                    .insert(index.value.clone(), chunk.pointer.clone())
                ;
            }
            let mut ptr = chunk.pointer.clone();
            ptr.drop_type();
            list_values.entry(chunk.pointer.type_name().to_string())
                .or_insert(Vec::new())
                .push(ptr);
        }
        
        let mut pointer_map : HashMap<String, PakUntypedPointer> = HashMap::new();
        for (key, tree) in map {
            let pointer = tree.into_pak(&mut self)?;
            pointer_map.insert(key, pointer.as_untyped());
        }
        
        let mut lists = HashMap::<String, PakPointer>::new();
        for (type_name, list) in list_values.into_iter() {
            let pointer = self.pak_no_search(list)?;
            lists.insert(type_name, pointer);
        }
        
        let meta = PakMeta {
            name: self.name,
            description: self.description,
            author: self.author,
            version: PAK_FILE_VERSION.to_string(),
        };
        
        let sizing = PakSizing {
            meta_size: bincode::serialized_size(&meta)?,
            indices_size: bincode::serialized_size(&pointer_map)?,
            vault_size: bincode::serialized_size(&self.vault)?,
            list_size: bincode::serialized_size(&lists)?,
        };
        
        let mut sizing_out = bincode::serialize(&sizing)?;
        let mut meta_out = bincode::serialize(&meta)?;
        let mut pointer_map_out = bincode::serialize(&pointer_map)?;
        let mut vault_out = bincode::serialize(&self.vault)?;
        let mut list_out = bincode::serialize(&lists)?;
        
        let mut out = Vec::<u8>::new();
        out.append(&mut sizing_out);
        out.append(&mut meta_out);
        out.append(&mut pointer_map_out);
        out.append(&mut list_out);
        out.append(&mut vault_out);
        Ok((out, sizing, meta))
    }
}

//==============================================================================================
//        PakVaultReference
//==============================================================================================

#[derive(Debug, Clone)]
pub(crate) struct PakVaultReference {
    pointer : PakPointer,
    indices : Vec<PakIndex>
}