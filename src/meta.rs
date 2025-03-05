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

#[cfg(test)]
mod test {
    use super::PakSizing;

    
    #[test]
    fn size_of_pak_sizes() {
        let sizing = PakSizing {
            meta_size: 0,
            indices_size: 0,
            vault_size: 0,
        };
        
        let size = bincode::serialized_size(&sizing).unwrap();
        assert_eq!(size, 24);
    }
    
}