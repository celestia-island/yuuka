use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod utils;

#[proc_macro]
pub fn derive_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as utils::derive_struct::DeriveStruct);

    let root_ident = input.ident;
    let mod_ident = syn::Ident::new(&format!("__{}", root_ident), root_ident.span());

    let structs = input.structs;
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

    let enums = input.enums;
    let enums = enums
        .iter()
        .map(|(k, v)| {
            let k = k;
            let v = v.iter().map(|(k, v)| {
                let k = k;
                let v = match v {
                    utils::derive_struct::EnumValue::Empty => {
                        quote! {
                            #k,
                        }
                    }
                    utils::derive_struct::EnumValue::Tuple(v) => {
                        let v = v.iter().map(|v| {
                            quote! {
                                #v,
                            }
                        });
                        quote! {
                            #k(#(#v)*),
                        }
                    }
                    utils::derive_struct::EnumValue::Struct(v) => {
                        let v = v.iter().map(|(k, v)| {
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
        mod #mod_ident {
            use super::*;

            #( #structs )*
            #( #enums )*
        }

        use #mod_ident::*;
    };
    ret.into()
}

#[proc_macro]
pub fn derive_struct_anonymously(input: TokenStream) -> TokenStream {
    // TODO: Implement derive_struct_anonymously macro
    input
}

#[proc_macro]
pub fn auto(input: TokenStream) -> TokenStream {
    // TODO: Implement auto macro
    input
}
