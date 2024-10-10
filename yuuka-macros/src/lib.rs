use proc_macro::TokenStream;
use quote::quote;
use std::{cell::RefCell, rc::Rc};
use syn::parse_macro_input;

mod utils;
use utils::{
    derive_struct::DeriveStructVisibility, flatten, DefaultValue, DeriveStruct, EnumValueFlatten,
    StructName,
};

#[proc_macro]
pub fn derive_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveStruct);
    dbg!(input.clone());

    let is_public = input.visibility == DeriveStructVisibility::Public;
    let root_ident = match input.ident.clone() {
        StructName::Named(v) => v,
        StructName::Unnamed(_) => {
            panic!("Unnamed root struct is not supported");
        }
    };
    let mod_ident = syn::Ident::new(&format!("__{}", root_ident), root_ident.span());
    let (structs, enums) = flatten(Rc::new(RefCell::new(0)), utils::DeriveBox::Struct(input))
        .expect("Failed to flatten");

    let structs = structs
        .iter()
        .map(|(ident, v, extra_macros)| {
            let keys = v
                .iter()
                .map(|(key, ty, _default_value)| {
                    quote! {
                        pub #key: #ty,
                    }
                })
                .collect::<Vec<_>>();
            let extra_macros = extra_macros
                .iter()
                .map(|content| {
                    quote! {
                        #[#content]
                    }
                })
                .collect::<Vec<_>>();

            if v.iter()
                .all(|(_, _, default_value)| default_value == &DefaultValue::None)
            {
                quote! {
                    #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, Default)]
                    #(#extra_macros)*
                    pub struct #ident {
                        #( #keys )*
                    }
                }
            } else {
                let default_values = v.iter()
                    .map(|(key, _ty, default_value)| match default_value {
                        DefaultValue::None => quote! {
                            #key: Default::default(),
                        },
                        DefaultValue::Single(v) => quote! {
                            #key: #v,
                        },
                        DefaultValue::Array(v) => quote! {
                            #key: vec![#(#v),*],
                        },
                    })
                    .collect::<Vec<_>>();

                quote! {
                    #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
                    pub struct #ident {
                        #( #keys )*
                    }

                    impl Default for #ident {
                        fn default() -> Self {
                            Self {
                                #( #default_values )*
                            }
                        }
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let enums = enums
        .iter()
        .map(|(k, v, default_value, extra_macros)| {
            let keys = v
                .iter()
                .map(|(key, ty)| match ty {
                    EnumValueFlatten::Empty => quote! {
                        #key,
                    },
                    EnumValueFlatten::Tuple(v) => {
                        quote! {
                            #key(#(#v),*),
                        }
                    }
                    EnumValueFlatten::Struct(v) => {
                        let keys = v
                            .iter()
                            .map(|(key, ty, _default_value)| {
                                quote! {
                                    #key: #ty,
                                }
                            })
                            .collect::<Vec<_>>();

                        quote! {
                            #key {
                                #( #keys )*
                            },
                        }
                    }
                })
                .collect::<Vec<_>>();
            let default_value = if let DefaultValue::Single(default_value) = default_value {
                quote! {
                    impl Default for #k {
                        fn default() -> Self {
                            #default_value
                        }
                    }
                }
            } else {
                quote! {
                    impl Default for #k {
                        fn default() -> Self {
                            unimplemented!("Default value for enum is not implemented");
                        }
                    }
                }
            };
            let extra_macros = extra_macros
                .iter()
                .map(|content| {
                    quote! {
                        #[#content]
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
                #(#extra_macros)*
                pub enum #k {
                    #( #keys )*
                }

                #default_value
            }
        })
        .collect::<Vec<_>>();

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
