use serde::{Deserialize, Serialize};
use strum_macros::Display;
use crate::{Pak, builder::PakBuilder, index::{PakIndex, PakIndexIdentifier}, item::PakItemSearchable, pointer::PakPointer, query::PakQuery, value::{IntoPakValue, PakValue}};

//==============================================================================================
//        Personallity Traits
//==============================================================================================

#[derive(Serialize, Deserialize, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum PersonalityTrait {
    Friendly,
    Intelligent,
    Curious,
    Brave,
    Kind,
    Creative,
    Adventurous,
    Responsible,
    Organized,
    Patient,
    Extroverted,
    Introverted,
}

impl IntoPakValue for PersonalityTrait {
    fn into_pak_value(self) -> PakValue {
        PakValue::String(self.to_string())
    }
}

//==============================================================================================
//        Person
//==============================================================================================

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
    pub age: u32,
    pub personallity_traits : Vec<PersonalityTrait>
}

impl PakItemSearchable for Person {
    fn get_indices(&self) -> Vec<PakIndex> {
        let mut indices = Vec::new();
        indices.push(PakIndex::new("first_name", self.first_name.clone()));
        indices.push(PakIndex::new("last_name", self.last_name.clone()));
        indices.push(PakIndex::new("age", self.age));
        indices.push(PakIndex::new("personallity_traits", self.personallity_traits.clone()));
        indices
    }
}

//==============================================================================================
//        Pet
//==============================================================================================

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Pet {
    pub name : String,
    pub age: u32,
    pub owner: PakPointer,
    pub kind: PetKind,
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

pub(crate) fn john_doe() -> Person {
    Person {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        age: 30,
        personallity_traits: vec![PersonalityTrait::Patient, PersonalityTrait::Creative, PersonalityTrait::Extroverted]
    }
}

pub(crate) fn jane_doe() -> Person {
    Person {
        first_name: "Jane".to_string(),
        last_name: "Doe".to_string(),
        age: 25,
        personallity_traits: vec![PersonalityTrait::Responsible, PersonalityTrait::Introverted, PersonalityTrait::Intelligent]
    }
}

pub(crate) fn alice_smith() -> Person {
    Person {
        first_name: "Alice".to_string(),
        last_name: "Smith".to_string(),
        age: 28,
        personallity_traits: vec![PersonalityTrait::Brave, PersonalityTrait::Adventurous, PersonalityTrait::Curious]
    }
}

pub(crate) fn bob_johnson() -> Person {
    Person {
        first_name: "Bob".to_string(),
        last_name: "Johnson".to_string(),
        age: 35,
        personallity_traits: vec![PersonalityTrait::Patient, PersonalityTrait::Creative, PersonalityTrait::Extroverted]
    }
}

pub(crate) fn charlie_brown() -> Person {
    Person {
        first_name: "Charlie".to_string(),
        last_name: "Brown".to_string(),
        age: 40,
        personallity_traits: vec![PersonalityTrait::Responsible, PersonalityTrait::Introverted, PersonalityTrait::Intelligent]
    }
}

pub(crate) fn john_jacob() -> Person {
    Person {
        first_name: "John".to_string(),
        last_name: "Jacob".to_string(),
        age: 45,
        personallity_traits: vec![PersonalityTrait::Patient, PersonalityTrait::Creative, PersonalityTrait::Extroverted]
    }
}

pub(crate) fn ajax_burnahm() -> Person {
    Person {
        first_name: "Ajax".to_string(),
        last_name: "Burnahm".to_string(),
        age: 30,
        personallity_traits: vec![PersonalityTrait::Responsible, PersonalityTrait::Introverted, PersonalityTrait::Intelligent]
    }
}

/// This is the unofficial build test, this runs in every test
pub(crate) fn build_data_base() -> (Pak, PakPointer, PakPointer) {
    let mut builder = PakBuilder::new();
    
    let person1 = john_doe();
    let person2 = jane_doe();
    let person3 = alice_smith();
    let person4 = bob_johnson();
    let person5 = charlie_brown();
    let person6 = john_jacob();
    let person7 = ajax_burnahm();
    
    let owner1 = builder.pak(person1).unwrap();
    let owner2 = builder.pak(person2).unwrap();
    builder.pak(person3).unwrap();
    builder.pak(person4).unwrap();
    builder.pak(person5).unwrap();
    builder.pak(person6).unwrap();
    builder.pak(person7).unwrap();
    
    let pet1 = Pet {
        name: "Fido".to_string(),
        age: 5,
        owner: owner1.clone(),
        kind: PetKind::Dog,
    };
    
    let pet2 = Pet {
        name: "Whiskers".to_string(),
        age: 3,
        owner: owner2.clone(),
        kind: PetKind::Cat,
    };
    
    let pet3 = Pet {
        name: "Bella".to_string(),
        age: 7,
        owner: owner1.clone(),
        kind: PetKind::Dog,
    };
    
    builder.pak(pet1).unwrap();
    builder.pak(pet2).unwrap();
    builder.pak(pet3).unwrap();
    
    (builder.build_in_memory().unwrap(), owner1, owner2)
}

#[test]
fn pak_read() {
    let (pak, john_doe, _) = build_data_base();
    let person = pak.read_err::<Person>(&john_doe).unwrap();
    
    assert_eq!(person.first_name, "John");
    assert_eq!(person.last_name, "Doe");
}

#[test]
fn pak_query_equal() {
    let (pak, _, _) = build_data_base();
    
    let people = pak.query::<(Person, )>("first_name".equals("John")).unwrap();
    assert_eq!(people.len(), 2);
}

#[test]
fn pak_query_less_than() {
    let (pak, _, _) = build_data_base();
    
    let (people, pets) = pak.query::<(Person, Pet)>("age".less_than_or_equal(26)).unwrap();
    
    assert_eq!(people.len(), 1);
    assert_eq!(pets.len(), 3);
}

#[test]
fn pak_query_greater_than() {
    let (pak, _, _) = build_data_base();
    
    let (people, pets) = pak.query::<(Person, Pet)>("age".greater_than(26)).unwrap();
    
    assert_eq!(people.len(), 6);
    assert_eq!(pets.len(), 0);
}

#[test]
fn pak_query_greater_than_equal() {
    let (pak, _, _) = build_data_base();
    
    let (people, pets) = pak.query::<(Person, Pet)>("age".greater_than_or_equal(25)).unwrap();
    
    assert_eq!(people.len(), 7);
    assert_eq!(pets.len(), 0);
}

#[test]
fn pak_query_less_than_equal() {
    let (pak, _, _) = build_data_base();
    
    let (people, pets) = pak.query::<(Person, Pet)>("age".less_than_or_equal(25)).unwrap();
    
    assert_eq!(people.len(), 1);
    assert_eq!(pets.len(), 3);
}

#[test]
fn pak_query_contains() {
    let (pak, _, _) = build_data_base();
    
    let query = "personallity_traits".contains_value(PersonalityTrait::Creative);
    let people = pak.query::<(Person,)>(query).unwrap();
    
    assert_eq!(people.len(), 3);
}

#[test]
fn pak_query_all() {
    let (pak, _, _) = build_data_base();
    
    let query = PakQuery::All;
    let (pets, people) = pak.query::<(Pet, Person)>(query).unwrap();
    
    assert_eq!(people.len(), 7);
    assert_eq!(pets.len(), 3);
}

#[test]
fn compound_union_query() {
    let (pak, _, _) = build_data_base();
    
    let query = "age".less_than(30) | "first_name".equals("John");
    let (people, pets) = pak.query::<(Person, Pet)>(query).unwrap();
    
    assert_eq!(people.len(), 4);
    assert_eq!(pets.len(), 3);
    
    assert!(people.contains(&john_doe()));
    assert!(people.contains(&john_jacob()));
    assert!(people.contains(&alice_smith()));
    assert!(people.contains(&jane_doe()));
}

#[test]
fn compound_intersection_query() {
    let (pak, _, _) = build_data_base();
    
    let query = "age".greater_than(25) & "first_name".equals("John");
    let (people, pets) = pak.query::<(Person, Pet)>(query).unwrap();
    
    assert_eq!(people.len(), 2);
    assert_eq!(pets.len(), 0);
}

#[test] 
fn pak_file_read_write() {
    let mut builder = PakBuilder::new();
    
    builder.pak(john_doe()).unwrap();
    builder.pak(jane_doe()).unwrap();
    builder.pak(alice_smith()).unwrap();
    builder.pak(bob_johnson()).unwrap();
    builder.pak(charlie_brown()).unwrap();
    builder.pak(john_jacob()).unwrap();
    builder.pak(ajax_burnahm()).unwrap();
    
    {
        let pak = builder.build_file("temp.pak").unwrap();
        
        let people = pak.query::<(Person, )>("first_name".equals("John")).unwrap();
        assert_eq!(people.len(), 2);
        assert!(people.contains(&john_doe()));
        assert!(people.contains(&john_jacob()));
    }
    
    let pak = Pak::new_from_file("temp.pak").unwrap();
    
    let people = pak.query::<(Person, )>("first_name".equals("John")).unwrap();
    assert_eq!(people.len(), 2);
    assert!(people.contains(&john_doe()));
    assert!(people.contains(&john_jacob()));
    
    std::fs::remove_file("temp.pak").unwrap();
}