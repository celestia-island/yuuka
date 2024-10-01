use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod utils;

#[proc_macro]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as utils::derive_config::DeriveConfig);

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
                    utils::derive_config::EnumValue::Empty => {
                        quote! {
                            #k,
                        }
                    }
                    utils::derive_config::EnumValue::Tuple(v) => {
                        let v = v.iter().map(|v| {
                            quote! {
                                #v,
                            }
                        });
                        quote! {
                            #k(#(#v)*),
                        }
                    }
                    utils::derive_config::EnumValue::Struct(v) => {
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
