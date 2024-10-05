use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod utils;
use utils::{derive_struct::DeriveStructVisibility, DeriveStruct, EnumValue};

#[proc_macro]
pub fn derive_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveStruct);

    let root_ident = input.ident.expect("Anonymous struct is not support yet.");
    let mod_ident = syn::Ident::new(&format!("__{}", root_ident), root_ident.span());

    let structs = input.sub_structs;
    let structs = structs
        .iter()
        .map(|(k, v)| {
            let k = k;
            let default_value_decl = v.iter().map(|(k, (_, default_value))| {
                let k = k;

                if let Some(default_value) = default_value {
                    quote! {
                        #k: #default_value,
                    }
                } else {
                    quote! {}
                }
            }).collect::<Vec<_>>();
            let v = v.iter().map(|(k, (v, _default_value))| {
                let k = k;
                let v = v;
                quote! {
                    pub #k: #v,
                }
            }).collect::<Vec<_>>();

            if default_value_decl.is_empty() {
                quote! {
                    #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, Default)]
                    pub struct #k {
                        #(#v)*
                    }
                }
            } else {
                quote! {
                    #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
                    pub struct #k {
                        #(#v)*
                    }

                    impl Default for #k {
                        fn default() -> Self {
                            Self {
                                #(#default_value_decl)*
                                ..Default::default()
                            }
                        }
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let enums = input.sub_enums;
    let enums = enums
        .iter()
        .map(|(k, (v, default_value))| {
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
                            .map(|(k, (v, _))| {
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

            if let Some(default_value) = default_value {
                quote! {
                    #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
                    pub enum #k {
                        #(#v)*
                    }

                    impl Default for #k {
                        fn default() -> Self {
                            Self::#default_value
                        }
                    }
                }
            } else {
                quote! {
                    #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
                    pub enum #k {
                        #(#v)*
                    }

                    impl Default for #k {
                        fn default() -> Self {
                            unimplemented!()
                        }
                    }
                }
            }
        })
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

    // TODO: Provide a macro_rules! at #mod_ident::auto! to generate the struct automatically
    ret.into()
}

#[proc_macro]
pub fn auto(input: TokenStream) -> TokenStream {
    // TODO: Implement auto macro that calls #mod_ident::auto!
    input
}
