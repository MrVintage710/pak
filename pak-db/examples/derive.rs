use pak_db::{index::PakIndexIdentifier, reference::PakItemRef, PakBuilder};
use pak_db_derive::PakItem;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PakItem, Debug)]
pub struct Hobby {
    #[index]
    name: String,
}

#[derive(Serialize, Deserialize, PakItem, Debug, Clone)]
pub struct Pet {
    #[index]
    name: String,
    #[index]
    age: u32,
}

#[derive(Serialize, Deserialize, PakItem, Debug)]
pub struct Person {
    #[index]
    name: String,
    #[index]
    age: u32,
    pets: Vec<PakItemRef<Pet>>
}

pub fn main() {
    let pet1 = Pet {
        name: "Sorin".to_string(),
        age: 2,
    };
    
    let pet2 = Pet {
        name: "Edgar".to_string(),
        age: 2,
    };
    
    let person = Person {
        name: "John".to_string(),
        age: 30,
        pets: vec![PakItemRef::Loaded(pet1), PakItemRef::Loaded(pet2)],
    };
    
    let mut builder = PakBuilder::new();
    builder.pak(person).unwrap();
    let pak = builder.build_in_memory().unwrap();
    let (people, pets) = pak.query::<(Person, Pet)>("name".less_than("M")).unwrap();
    
    println!("{people:?}\n-------------------\n{pets:?}")
}
