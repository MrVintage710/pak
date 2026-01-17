use std::{fs::File, io::BufReader, path::Path};

use serde::{Deserialize, Serialize};

use crate::{PAK_SIZING_STRUCT_SIZE_IN_BYTES, PakSource, error::PakResult, pointer::PakPointer};

/// The metadata for a Pak file. Each pak file has this data embedded within the header.
#[derive(Serialize, Deserialize)]
pub struct PakMeta {
    pub name: String,
    pub version: String,
    pub pak_version : String,
    pub description: String,
    pub author: String,
    pub identifier : String,
    pub(crate) extra : Vec<u8>,
}

impl PakMeta {
    /// This returns the extra data that can be saved in the metadata. This can throw an error if the
    /// wrong type is asked for.
    pub fn get_extra<T>(&self) -> PakResult<T> where T : for<'de> Deserialize<'de> {
        Ok(bincode::deserialize(&self.extra)?)
    }
    
    /// Reads the Metadata of a pak file that is passed into the function
    pub fn read_meta(file : impl AsRef<Path>) -> PakResult<PakMeta> {
        let mut source = BufReader::new(File::open(file)?);
        
        let sizing_pointer = PakPointer::new_untyped(0, PAK_SIZING_STRUCT_SIZE_IN_BYTES);
        let sizing_buffer = source.read(&sizing_pointer, 0)?;
        let sizing : PakSizing = bincode::deserialize(&sizing_buffer)?;
        
        let meta_pointer = PakPointer::new_untyped(PAK_SIZING_STRUCT_SIZE_IN_BYTES, sizing.meta_size);
        let meta_buffer = source.read(&meta_pointer, 0)?;
        let meta : PakMeta = bincode::deserialize(&meta_buffer)?;
        
        Ok(meta)
    }
}

/// This carries the size information of each part of the Pak file. this is always the first 32 bytes of the file.
#[derive(Serialize, Deserialize, Debug)]
pub struct PakSizing {
    pub meta_size: u64,
    pub indices_size: u64,
    pub vault_size: u64,
    pub list_size: u64,
}

impl PakSizing {
    /// Reads the Sizing struct of a pak file that is passed into the function
    pub fn read_sizing(file : impl AsRef<Path>) -> PakResult<PakSizing> {
        let mut source = BufReader::new(File::open(file)?);
        
        let sizing_pointer = PakPointer::new_untyped(0, PAK_SIZING_STRUCT_SIZE_IN_BYTES);
        let sizing_buffer = source.read(&sizing_pointer, 0)?;
        let sizing : PakSizing = bincode::deserialize(&sizing_buffer)?;
        
        Ok(sizing)
    }
}