use serde::{Deserialize, Serialize};

/// The metadata for a Pak file. Each pak file has this data embedded within the header.
#[derive(Serialize, Deserialize)]
pub struct PakMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

/// This carries the size information of each part of the Pak file. this is always the first 24 bytes of the file.
#[derive(Serialize, Deserialize, Debug)]
pub struct PakSizing {
    pub meta_size: u64,
    pub indices_size: u64,
    pub vault_size: u64,
}