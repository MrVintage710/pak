use std::{collections::HashSet, ops::{BitAnd, BitOr}};

use crate::error::PakResult;

use super::{value::PakValue, Pak, PakPointer};

//==============================================================================================
//        Pak Query
//==============================================================================================

pub trait PakQueryExpression {
    fn execute(&self, pak : &Pak) -> PakResult<HashSet<PakPointer>>;
}

pub struct PakQueryUnion(Box<dyn PakQueryExpression>, Box<dyn PakQueryExpression>);

impl PakQueryExpression for PakQueryUnion {
    fn execute(&self, pak : &Pak) -> PakResult<HashSet<PakPointer>> {
        let results_a = self.0.execute(pak)?;
        let results_b = self.1.execute(pak)?;
        println!("UNION: {results_a:?} AND {results_b:?}");
        let results = results_a.into_iter().chain(results_b.into_iter()).collect::<HashSet<_>>();
        Ok(results)
    }
}

impl<B> BitOr<B> for PakQueryUnion where B : PakQueryExpression + 'static {
    type Output = Self;

    fn bitor(self, other: B) -> Self::Output {
        PakQueryUnion(Box::new(self), Box::new(other))
    }
}

impl <B> BitOr<B> for PakQueryIntersection where B : PakQueryExpression + 'static {
    type Output = PakQueryUnion;

    fn bitor(self, other: B) -> Self::Output {
        PakQueryUnion(Box::new(self), Box::new(other))
    }
}

impl <B> BitOr<B> for PakQuery where B : PakQueryExpression + 'static {
    type Output = PakQueryUnion;

    fn bitor(self, other: B) -> Self::Output {
        PakQueryUnion(Box::new(self), Box::new(other))
    }
}

//==============================================================================================
//        Pak Query Intersection
//==============================================================================================

pub struct PakQueryIntersection(Box::<dyn PakQueryExpression>, Box::<dyn PakQueryExpression>);

impl PakQueryExpression for PakQueryIntersection {
    fn execute(&self, pak : &Pak) -> PakResult<HashSet<PakPointer>> {
        let results_a = self.0.execute(pak)?;
        let results_b = self.1.execute(pak)?;
        println!("INTERSECTION: {results_a:?} AND {results_b:?}");
        Ok(results_a.into_iter().filter(|e| results_b.contains(e)).collect())
    }
}

impl <B> BitAnd<B> for PakQuery where B : PakQueryExpression + 'static {
    type Output = PakQueryIntersection;

    fn bitand(self, rhs: B) -> Self::Output {
        PakQueryIntersection(Box::new(self), Box::new(rhs))
    }
}

impl <B> BitAnd<B> for PakQueryUnion where B : PakQueryExpression + 'static {
    type Output = PakQueryIntersection;

    fn bitand(self, rhs: B) -> Self::Output {
        PakQueryIntersection(Box::new(self), Box::new(rhs))
    }
}

impl <B> BitAnd<B> for PakQueryIntersection where B : PakQueryExpression + 'static {
    type Output = PakQueryIntersection;

    fn bitand(self, rhs: B) -> Self::Output {
        PakQueryIntersection(Box::new(self), Box::new(rhs))
    }
}


//==============================================================================================
//        Pak Query Expression
//==============================================================================================

pub enum PakQuery {
    Equal(String, PakValue),
    GreaterThan(String, PakValue),
    LessThan(String, PakValue),
}

impl PakQuery {
    pub fn equals(key : &str, value : impl Into<PakValue>) -> Self {
        PakQuery::Equal(key.to_string(), value.into())
    }

    pub fn greater_than(key : &str, value : impl Into<PakValue>) -> Self {
        PakQuery::GreaterThan(key.to_string(), value.into())
    }

    pub fn less_than(key : &str, value : impl Into<PakValue>) -> Self {
        PakQuery::LessThan(key.to_string(), value.into())
    }
}

pub fn equals(key : &str, value : impl Into<PakValue>) -> PakQuery {
    PakQuery::Equal(key.to_string(), value.into())
}

pub fn greater_than(key : &str, value : impl Into<PakValue>) -> PakQuery {
    PakQuery::GreaterThan(key.to_string(), value.into())
}

pub fn less_than(key : &str, value : impl Into<PakValue>) -> PakQuery {
    PakQuery::LessThan(key.to_string(), value.into())
}

impl PakQueryExpression for PakQuery {
    fn execute(&self, pak : &Pak) -> PakResult<HashSet<PakPointer>> {
        match self {
            PakQuery::Equal(key, pak_value) => {
                let tree = pak.get_tree(key)?;
                tree.get(pak_value)
            },
            PakQuery::GreaterThan(key, pak_value) => {
                let tree = pak.get_tree(key)?;
                tree.get_greater(pak_value)
            },
            PakQuery::LessThan(key, pak_value) => {
                let tree = pak.get_tree(key)?;
                tree.get_less(pak_value)
            },
        }
    }
}

//==============================================================================================
//        Tests
//==============================================================================================

#[cfg(test)]
mod tests {
    use std::sync::Once;
    use serde::{Deserialize, Serialize};
    use crate::{index::PakIndex, item::PakItemSearchable, query::*, Pak, PakBuilder};
    
    static INIT: Once = Once::new();
    
    pub fn initialize() {
        INIT.call_once(|| {
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
                first_name: "John".to_string(),
                last_name: "Jacob".to_string(),
                age: 28,
            };
            
            builder.pak(person1).unwrap();
            builder.pak(person2).unwrap();
            builder.pak(person3).unwrap();
            builder.pak(person4).unwrap();
            
            builder.build("test.pak").unwrap();
        });
    }
    
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

    #[test]
    fn query() {
        initialize();
        
        let pak = Pak::open("test.pak").unwrap();
        
        let query = greater_than("age", 26) & equals("first_name", "John");
        
        let results = pak.query::<Person>(query).unwrap();
        println!("RESULTS {results:?}");
        assert_eq!(results.len(), 2);
    }
}