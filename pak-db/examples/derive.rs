use pak_db::item::PakItemRef;
use pak_db_derive::PakItem;

#[derive(PakItem)]
pub struct Pet {
    name: String,
    age: u32,
}

#[derive(PakItem)]
pub struct Person {
    #[index]
    name: String,
    age: u32,
    pets: Vec<PakItemRef<Pet>>
}

pub fn main() {
    println!("Hello world");
}
