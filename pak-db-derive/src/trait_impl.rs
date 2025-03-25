use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Data, DataStruct, Fields, FieldsNamed, Ident, Type};

//==============================================================================================
//        FieldInfo
//==============================================================================================

struct FieldInfo {
    name: Ident,
    span : Span,
    index : bool
}

//==============================================================================================
//        functions
//==============================================================================================

///This is the main method responsible for implmenting the PakItem trait for a struct/enum.
pub fn impl_pak_item(name : &Ident, data : &Data) -> TokenStream {
    let fields = match data {
        Data::Struct(data_struct) => impl_pak_item_struct(data_struct),
        Data::Enum(data_enum) => unimplemented!(),
        _ => unimplemented!()
    };
    
    let fields_as_prelude_calls = fields.iter().map(|f| {
        let field_name = &f.name;
        let field_span = f.span;
        quote_spanned! { field_span => ::pak_db::prelude::PakItem::prelude(&mut self.#field_name, builder)?}
    }).collect::<Vec<_>>();
    
    let fields_as_byte_list = fields.iter().map(|f| {
        let field_name = &f.name;
        let field_span = f.span;
        quote_spanned! { field_span => ::pak_db::prelude::serialize(&self.#field_name)?}
    }).collect::<Vec<_>>();
    
    let mut index : usize = 0;
    let field_from_bytes_list = fields.iter().map(|f| {
        let field_name = &f.name;
        let field_span = f.span;
        let stream = quote_spanned! { field_span => #field_name: ::pak_db::prelude::deserialize(&bytes[#index])?};
        index += 1;
        stream
    }).collect::<Vec<_>>();
    
    
    let field_to_indices_list = fields.iter().filter(|f| f.index).map(|f| {
        let field_name = &f.name;
        let field_name_str = field_name.to_string();
        let field_span = f.span;
        quote_spanned! {field_span => ::pak_db::index::PakIndex::new(#field_name_str, self.#field_name.clone())}
    }).collect::<Vec<_>>();
    
    if field_to_indices_list.len() == 0 {
        panic!("There are no fields labeled as an 'index'. This will make the item unsearchable.")
    }
    
    // fn pak(mut self, builder : &mut pak_db::prelude::PakBuilder) -> pak_db::prelude::PakResult<pak_db::prelude::PakPointer> {
    //     let bytes = vec![#(#fields_as_byte_list),*];
    //     let indices = self.indices();
    //     let pointer = builder.store_multiple(bytes, indices)?;
    //     return Ok(pointer)
    // }
    
    // fn unpak(pak : & pak_db::prelude::Pak, pointer : & pak_db::prelude::PakPointer) -> pak_db::prelude::PakResult<Self> {
    //     let bytes : Vec<Vec<u8>> = pak.read_chunk(pointer)?;
    //     let result = Self {
    //         #(#field_from_bytes_list),*
    //     };
    //     Ok(result)
    // }
    
    quote! {
        impl ::pak_db::prelude::PakItem for #name {
            fn pak(mut self, builder : &mut pak_db::prelude::PakBuilder) -> pak_db::prelude::PakResult<pak_db::prelude::PakPointer> {
                let bytes = pak_db::prelude::serialize(&self)?;
                let indices = self.indices();
                let pointer = builder.store::<Self>(bytes, indices)?;
                return Ok(pointer)
            }
            
            fn prelude(&mut self, builder : &mut PakBuilder) -> pak_db::prelude::PakResult<()> {
                #(#fields_as_prelude_calls;)*
                Ok(())
            }
            
            fn unpak(pak : & pak_db::prelude::Pak, pointer : & pak_db::prelude::PakPointer) -> pak_db::prelude::PakResult<Self> {
                pak.read_err::<Self>(pointer)
            }
            
            fn indices(&self) -> Vec<pak_db::prelude::PakIndex> {
                vec![#(#field_to_indices_list),*]
            }
        }
    }
}

fn impl_pak_item_struct(data : &DataStruct) -> Vec<FieldInfo>  {
    match &data.fields {
        Fields::Named(fields_named) => impl_pak_item_struct_named_fields(fields_named),
        Fields::Unnamed(fields_unnamed) => unimplemented!(),
        Fields::Unit => unimplemented!(),
    }
}

fn impl_pak_item_struct_named_fields(fields : &FieldsNamed) -> Vec<FieldInfo> {
    
    let mut field_info = vec![];
    
    for field in fields.named.iter() {
        let index = field.attrs.iter().any(|attr| attr.meta.path().is_ident("index"));
        let name = field.ident.as_ref().unwrap().clone();
        let span = field.span();
        field_info.push(FieldInfo { name, span, index });
        // let stream = quote_spanned! { field.span() => ::pak_db::prelude::IntoBytes::into_bytes(&mut self.#iden, builder)};
        // field_info.push(stream);
    }
    
    field_info
}