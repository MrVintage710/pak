use serde::{de::DeserializeOwned, Serialize};
use crate::error::PakResult;
use super::{index::PakIndex, PakPointer};

//==============================================================================================
//        PakItem Trait
//==============================================================================================

pub trait PakItemSearchable {
    fn get_indices(&self) -> Vec<PakIndex>;
}

pub trait PakItemDef {
    fn into_bytes(&self) -> PakResult<Vec<u8>>;
}

pub trait PakItemRef: Sized {
    fn from_bytes(bytes: &[u8]) -> PakResult<Self>;
    
    fn from_pak(pak : &[u8], pointer : PakPointer) -> PakResult<Self> {
        let data = &pak[pointer.offset as usize..pointer.offset as usize + pointer.size as usize];
        let res = Self::from_bytes(data)?;
        Ok(res)
    }
}

impl <T> PakItemRef for T where T : DeserializeOwned {
    fn from_bytes(bytes: &[u8]) -> PakResult<Self> {
        let obj : Self = bincode::deserialize::<Self>(bytes)?;
        Ok(obj)
    }
}

impl <T> PakItemDef for T where T : Serialize {
    fn into_bytes(&self) -> PakResult<Vec<u8>> {
        bincode::serialize(self).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod test {

    use serde::{Deserialize, Serialize};
    use crate::PakBuilder;

    use super::*;
    
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    struct Person {
        first_name: String,
        last_name: String,
    }
    
    impl PakItemSearchable for Person {
        fn get_indices(&self) -> Vec<PakIndex> {
            vec![
                PakIndex::new("first_name", self.first_name.clone()),
                PakIndex::new("last_name", self.last_name.clone())
            ]
        }
    }
    
    #[test]
    fn pak_vault() {
        let person1 = Person {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
        };
        
        let person2 = Person {
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
        };
        
        let mut vault = PakBuilder::new();
        vault.pak(person1.clone()).unwrap();
        vault.pak(person2.clone()).unwrap();
        
        assert_eq!(
            vault.vault,
            vec![
                bincode::serialize(&person1).unwrap(),
                bincode::serialize(&person2).unwrap()
            ].iter().cloned().flatten().collect::<Vec<u8>>()
        );
    }
    
    #[test]
    fn unpak_vault() {
        let person1 = Person {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
        };
        
        let person2 = Person {
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
        };
        
        let mut vault = PakBuilder::new();
        let reference1 = vault.pak(person1.clone()).unwrap();
        let reference2 = vault.pak(person2.clone()).unwrap();
        
        let person1_unpak : Person = vault.unpak(&reference1.pointer).unwrap();
        let person2_unpak : Person = vault.unpak(&reference2.pointer).unwrap();
        
        assert_eq!(person1, person1_unpak);
        assert_eq!(person2, person2_unpak);
    }
}
