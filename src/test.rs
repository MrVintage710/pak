use serde::{Deserialize, Serialize};
use crate::{index::{PakIndex, PakIndexIdentifier}, item::PakItemSearchable, pointer::PakPointer, value::IntoPakValue, Pak, PakBuilder};

//==============================================================================================
//        Person
//==============================================================================================

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct Person {
    first_name: String,
    last_name: String,
    age: u32,
}

impl PakItemSearchable for Person {
    fn get_indices(&self) -> Vec<PakIndex> {
        let mut indices = Vec::new();
        indices.push(PakIndex::new("first_name", self.first_name.clone()));
        indices.push(PakIndex::new("last_name", self.last_name.clone()));
        indices.push(PakIndex::new("age", self.age));
        indices
    }
}

//==============================================================================================
//        Pet
//==============================================================================================

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct Pet {
    name : String,
    age: u32,
    owner: PakPointer,
    kind: PetKind,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PetKind {
    Dog,
    Cat,
}

impl IntoPakValue for PetKind {
    fn into_pak_value(self) -> crate::value::PakValue {
        match self {
            PetKind::Dog => "dog".into(),
            PetKind::Cat => "cat".into(),
        }
    }
}

impl PakItemSearchable for Pet {
    fn get_indices(&self) -> Vec<PakIndex> {
        let mut indices = Vec::new();
        indices.push(PakIndex::new("name", self.name.clone()));
        indices.push(PakIndex::new("age", self.age));
        indices.push(PakIndex::new("kind", self.kind.clone()));
        indices
    }
}

/// This is the unofficial build test, this runs in every test
pub fn build_data_base() -> Pak {
    let mut builder = PakBuilder::new();
    
    let person1 = Person {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        age: 30,
    };
    
    let person2 = Person {
        first_name: "Jane".to_string(),
        last_name: "Doe".to_string(),
        age: 25,
    };
    
    let person3 = Person {
        first_name: "Alice".to_string(),
        last_name: "Smith".to_string(),
        age: 28,
    };
    
    let person4 = Person {
        first_name: "Bob".to_string(),
        last_name: "Johnson".to_string(),
        age: 35,
    };
    
    let person5 = Person {
        first_name: "Charlie".to_string(),
        last_name: "Brown".to_string(),
        age: 40,
    };
    
    let person6 = Person {
        first_name: "John".to_string(),
        last_name: "Jacob".to_string(),
        age: 45,
    };
    
    
    let owner1 = builder.pak(person1).unwrap();
    let owner2 = builder.pak(person2).unwrap();
    builder.pak(person3).unwrap();
    builder.pak(person4).unwrap();
    builder.pak(person5).unwrap();
    builder.pak(person6).unwrap();
    
    let pet1 = Pet {
        name: "Fido".to_string(),
        age: 5,
        owner: owner1.clone(),
        kind: PetKind::Dog,
    };
    
    let pet2 = Pet {
        name: "Whiskers".to_string(),
        age: 3,
        owner: owner2,
        kind: PetKind::Cat,
    };
    
    let pet3 = Pet {
        name: "Bella".to_string(),
        age: 7,
        owner: owner1,
        kind: PetKind::Dog,
    };
    
    builder.pak(pet1).unwrap();
    builder.pak(pet2).unwrap();
    builder.pak(pet3).unwrap();
    
    builder.build_in_memory().unwrap()
}

#[test]
fn pak_read() {
    let pak = build_data_base();
    let person : Person = pak.read_err(&PakPointer::new_untyped(0, 27)).unwrap();
    
    assert_eq!(person.first_name, "John");
    assert_eq!(person.last_name, "Doe");
}

#[test]
fn pak_query_equal() {
    let pak = build_data_base();
    
    let people = pak.query::<(Person, )>("first_name".equals("John")).unwrap();
    assert_eq!(people.len(), 2);
}

#[test]
fn pak_query_less_than() {
    let pak = build_data_base();
    
    let (people, pets) = pak.query::<(Person, Pet)>("age".less_than_or_equal(26)).unwrap();
    
    assert_eq!(people.len(), 1);
    assert_eq!(pets.len(), 3);
}

#[test]
fn pak_query_greater_than() {
    let pak = build_data_base();
    
    let (people, pets) = pak.query::<(Person, Pet)>("age".greater_than(26)).unwrap();
    
    assert_eq!(people.len(), 5);
    assert_eq!(pets.len(), 0);
}

#[test]
fn pak_query_greater_than_equal() {
    let pak = build_data_base();
    
    let (people, pets) = pak.query::<(Person, Pet)>("age".greater_than_or_equal(25)).unwrap();
    
    assert_eq!(people.len(), 6);
    assert_eq!(pets.len(), 0);
}

#[test]
fn pak_query_less_than_equal() {
    let pak = build_data_base();
    
    let (people, pets) = pak.query::<(Person, Pet)>("age".less_than_or_equal(25)).unwrap();
    
    assert_eq!(people.len(), 1);
    assert_eq!(pets.len(), 3);
}

#[test]
fn compound_union_query() {
    let pak = build_data_base();
    
    let query = "age".less_than(30) | "first_name".equals("John");
    let (people, pets) = pak.query::<(Person, Pet)>(query).unwrap();
    
    assert_eq!(people.len(), 4);
    assert_eq!(pets.len(), 3);
}

#[test]
fn compound_intersection_query() {
    let pak = build_data_base();
    
    let query = "age".greater_than(25) & "first_name".equals("John");
    let (people, pets) = pak.query::<(Person, Pet)>(query).unwrap();
    
    assert_eq!(people.len(), 2);
    assert_eq!(pets.len(), 0);
}
