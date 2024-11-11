use proc_macro::TokenStream;
use quote::quote;
use std::{cell::RefCell, rc::Rc};
use syn::parse_macro_input;

mod template;
mod tools;

use template::{generate_enums_quote, generate_structs_quote};
use tools::{flatten, DeriveEnum, DeriveStruct, DeriveVisibility, StructName};

#[proc_macro]
pub fn derive_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveStruct);

    let is_public = input.visibility == DeriveVisibility::Public;
    let root_ident = match input.ident.clone() {
        StructName::Named(v) => v,
        StructName::Unnamed(_) => {
            panic!("Unnamed root struct is not supported");
        }
    };
    let mod_ident = syn::Ident::new(&format!("__{}", root_ident), root_ident.span());
    let (structs, enums) = flatten(
        root_ident.to_string(),
        Rc::new(RefCell::new(0)),
        tools::DeriveBox::Struct(input.clone()),
    )
    .expect("Failed to flatten");

    let structs = generate_structs_quote(structs);
    let enums = generate_enums_quote(enums);

    let ret = if is_public {
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

#[proc_macro]
pub fn derive_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveEnum);

    let is_public = input.visibility == DeriveVisibility::Public;
    let root_ident = match input.ident.clone() {
        StructName::Named(v) => v,
        StructName::Unnamed(_) => {
            panic!("Unnamed root struct is not supported");
        }
    };
    let mod_ident = syn::Ident::new(&format!("__{}", root_ident), root_ident.span());
    let (structs, enums) = flatten(
        root_ident.to_string(),
        Rc::new(RefCell::new(0)),
        tools::DeriveBox::Enum(input.clone()),
    )
    .expect("Failed to flatten");

    let structs = generate_structs_quote(structs);
    let enums = generate_enums_quote(enums);

    let ret = if is_public {
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
