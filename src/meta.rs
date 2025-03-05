use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PakMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PakSizing {
    pub meta_size: u64,
    pub indices_size: u64,
    pub vault_size: u64,
}