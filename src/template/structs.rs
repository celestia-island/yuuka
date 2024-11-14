use proc_macro2::TokenStream;
use quote::quote;

use crate::tools::{DefaultValue, StructsFlatten};

pub(crate) fn generate_structs_quote(structs: StructsFlatten) -> Vec<TokenStream> {
    structs
        .iter()
        .map(|(ident, v, extra_macros)| {
            let keys = v
                .iter()
                .map(|(key, ty, _default_value, extra_macros)| {
                    let extra_macros = extra_macros
                        .iter()
                        .map(|content| {
                            quote! {
                                #[#content]
                            }
                        })
                        .collect::<Vec<_>>();

                    quote! {
                        #(#extra_macros)*
                        pub #key: #ty,
                    }
                })
                .collect::<Vec<_>>();

            let derive_macros = extra_macros.derive_macros.clone();
            let attr_macros = extra_macros.attr_macros.clone();

            let derive_macros = if derive_macros.is_empty() {
                quote! {}
            } else {
                quote! {
                    #[derive(#(#derive_macros),*)]
                }
            };
            let attr_macros = if attr_macros.is_empty() {
                quote! {}
            } else {
                let list = attr_macros
                    .iter()
                    .map(|content| {
                        quote! {
                           #[#content]
                        }
                    })
                    .collect::<Vec<_>>();
                quote! {
                    #(#list)*
                }
            };

            if v.iter()
                .all(|(_, _, default_value, _)| default_value == &DefaultValue::None)
            {
                quote! {
                    #[derive(Debug, Clone, Default)]
                    #derive_macros
                    #attr_macros
                    pub struct #ident {
                        #( #keys )*
                    }
                }
            } else {
                let default_values = v
                    .iter()
                    .map(|(key, _, default_value, _)| match default_value {
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
                    #[derive(Debug, Clone)]
                    #derive_macros
                    #attr_macros
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
        .collect::<Vec<_>>()
}
