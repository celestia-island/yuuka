use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod utils;
use utils::{
    derive_struct::DeriveStructVisibility, DefaultValue, DeriveStruct, EnumValue, StructName,
};

#[proc_macro]
pub fn derive_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveStruct);
    dbg!(input.clone());

    let root_ident = match input.ident.clone() {
        StructName::Named(v) => v,
        StructName::Unnamed(_) => {
            panic!("Unnamed root struct is not supported");
        }
    };
    let mod_ident = syn::Ident::new(&format!("__{}", root_ident), root_ident.span());

    let structs = input.sub_structs;
    let structs = structs
        .iter()
        .map(|(key_raw, v)| {
            let key = key_raw.to_ident().expect("Invalid struct name");
            let default_value_decl = v.iter().map(|(key, _ty, default_value)| {
                match default_value {
                    DefaultValue::None => {
                        quote! {
                            #key: Default::default(),
                        }
                    }
                    DefaultValue::Single(v) => {
                        quote! {
                            #key: #v,
                        }
                    }
                    DefaultValue::Array(v) => {
                        match key_raw {
                            StructName::Unnamed(_) => {
                                let v = v.iter().map(|v| {
                                    quote! {
                                        #key::#v
                                    }
                                }).collect::<Vec<_>>();
                                quote! {
                                    #key: vec![#(#v),*],
                                }
                            }
                            StructName::Named(_) => {
                                quote! {
                                    #key: vec![#(#v),*],
                                }
                            }
                        }
                    }
                }
            }).collect::<Vec<_>>();
            let v = v.iter().map(|(key, ty, _default_value)| {
                let ty = ty.to_type_path().expect("Invalid type path");
                quote! {
                    pub #key: #ty,
                }
            }).collect::<Vec<_>>();

            if default_value_decl.is_empty() {
                quote! {
                    #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, Default)]
                    pub struct #key {
                        #(#v)*
                    }
                }
            } else if default_value_decl.len() != v.len() {
                quote! {
                    #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
                    pub struct #key {
                        #(#v)*
                    }

                    impl Default for #key {
                        fn default() -> Self {
                            Self {
                                #(#default_value_decl)*
                                ..Default::default()
                            }
                        }
                    }
                }
            } else {
                quote! {
                    #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
                    pub struct #key {
                        #(#v)*
                    }

                    impl Default for #key {
                        fn default() -> Self {
                            Self {
                                #(#default_value_decl)*
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
        .map(|(k_raw, (v, default_value))| {
            let k = k_raw.to_ident().expect("Invalid struct name");
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
                            let v = v.to_type_path().expect("Invalid type path");
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
                            .map(|(key, ty, _default_value)| {
                                let ty = ty.to_type_path().expect("Invalid type path");
                                quote! {
                                    #key: #ty,
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

    ret.into()
}
