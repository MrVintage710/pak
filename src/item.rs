use serde::{Deserialize, Serialize};

use crate::{PakInner, builder::PakBuilder, error::PakResult, index::{Indices, PakSearchable}, pointer::PakPointer};

pub trait PakSerialize {   
    fn pak(&self, pak : &mut PakBuilder) -> PakResult<PakPointer>;
}

impl <T> PakSerialize for T where T : Serialize + PakSearchable {
   fn pak(&self, packer : &mut PakBuilder) -> PakResult<PakPointer> {
        let mut indices = Indices::default();
        self.get_indices(&mut indices);
        packer.pak_serde::<T>(self, indices)
    }
}

pub trait PakDeserialize : Sized {
    fn unpak(pak : &PakInner, pointer : &PakPointer) -> PakResult<Self>;
}

impl <T> PakDeserialize for T where T : for<'de> Deserialize<'de> {
    fn unpak(pak : &PakInner, pointer : &PakPointer) -> PakResult<Self> {
        pak.read_serde(pointer)
    }
} 