use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod utils;
use utils::{DeriveStruct, EnumValue};

#[proc_macro]
pub fn derive_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as utils::derive_struct::DeriveStruct);

    let root_ident = input.ident.expect("Anonymous struct is not support yet.");
    let mod_ident = syn::Ident::new(&format!("__{}", root_ident), root_ident.span());

    let structs = input.sub_structs;
    let structs = structs
        .iter()
        .map(|(k, v)| {
            let k = k;
            let v = v.iter().map(|(k, v)| {
                let k = k;
                let v = v;
                quote! {
                    pub #k: #v,
                }
            });
            quote! {
                #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
                pub struct #k {
                    #(#v)*
                }
            }
        })
        .collect::<Vec<_>>();

    let enums = input.sub_enums;
    let enums = enums
        .iter()
        .map(|(k, v)| {
            let k = k;
            let v = v.iter().map(|(k, v)| {
                let k = k;
                let v = match v {
                    EnumValue::Empty => {
                        quote! {
                            #k,
                        }
                    }
                    EnumValue::Tuple(v) => {
                        let v = v.iter().map(|v| {
                            quote! {
                                #v,
                            }
                        });
                        quote! {
                            #k(#(#v)*),
                        }
                    }
                    EnumValue::Struct(ident) => {
                        let v = ident
                            .iter()
                            .map(|(k, v)| {
                                let k = k;
                                let v = v;
                                quote! {
                                    #k: #v,
                                }
                            })
                            .collect::<Vec<_>>();
                        quote! {
                            #k { #(#v)* },
                        }
                    }
                };
                v
            });
            quote! {
                #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
                pub enum #k {
                    #(#v)*
                }
            }
        })
        .collect::<Vec<_>>();

    let ret = quote! {
        #[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
        pub mod #mod_ident {
            use super::*;

            #( #structs )*
            #( #enums )*
        }

        pub use #mod_ident::*;
    };
    ret.into()
}

#[proc_macro]
pub fn auto(input: TokenStream) -> TokenStream {
    // TODO: Implement auto macro
    input
}
