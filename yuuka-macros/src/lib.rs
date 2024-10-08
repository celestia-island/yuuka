use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod utils;
use utils::{derive_struct::DeriveStructVisibility, flatten, DeriveStruct, StructName};

#[proc_macro]
pub fn derive_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveStruct);
    dbg!(input.clone());

    let root_ident = match input.ident.clone() {
        StructName::Named(v) => v,
        StructName::Unnamed => {
            panic!("Unnamed root struct is not supported");
        }
    };
    let mod_ident = syn::Ident::new(&format!("__{}", root_ident), root_ident.span());
    let (structs, enums) = flatten(input).expect("Failed to flatten");

    let structs = structs
        .iter()
        .map(|(k, v, default_value)| todo!())
        .collect::<Vec<_>>();

    let enums = enums
        .iter()
        .map(|(k, v, default_value)| todo!())
        .collect::<Vec<_>>();

    let ret = if input.visibility == DeriveStructVisibility::Public {
        quote! {
            #[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
            pub mod #mod_ident {
                use super::*;

                #( #structs )*
                #( #enums )*
            }

            pub use #mod_ident::*;
        }
    } else {
        quote! {
            #[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
            pub(crate) mod #mod_ident {
                use super::*;

                #( #structs )*
                #( #enums )*
            }

            pub(crate) use #mod_ident::*;
        }
    };

    ret.into()
}
