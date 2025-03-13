use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, spanned::Spanned, token::Comma, Data, DeriveInput, Fields, Ident, Variant};

#[proc_macro_derive(PakItem)]
pub fn pak_item_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input_meta = parse_macro_input!(input as DeriveInput);
    
    // Used in the quasi-quotation below as `#name`.
    let name = input_meta.ident;
    
    let enum_def = impl_iden_enum(&name, &input_meta.data);
    
    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        #enum_def        
    };

    // Hand the output tokens back to the compiler
    proc_macro::TokenStream::from(expanded)
}

///This method takes in the derived struct and return the tokenstream on the enum for it's Ids.
fn impl_iden_enum(name : &Ident, data : &Data) -> TokenStream {
    let new_ident = Ident::new(&format!("{}Field", name), name.span());
    
    let internal_tokens = impl_id_enum_internal_tokens(data);
    quote! {
        #[allow(non_camel_case_types)]
        pub enum #new_ident {
            #internal_tokens
        }
    }
}

fn impl_id_enum_internal_tokens(data : &Data) -> Punctuated<Variant, Comma> {
    match data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields_named) => {
                    let mut list = Punctuated::new();
                    for field in fields_named.named.iter() {
                        let is_not_searchable = field.attrs.iter().any(|attr| attr.meta.path().is_ident("not_searchable"));
                        if is_not_searchable { continue; }
                        let name = Ident::new(&format!("{}", field.ident.as_ref().unwrap()), field.span());
                        let variant = Variant {
                            attrs : Vec::new(),
                            ident : name,
                            fields : Fields::Unit,
                            discriminant : None,
                        };
                        list.push(variant);
                    }
                    list
                },
                Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
            }
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!()
    }
}