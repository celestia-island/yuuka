use proc_macro2::TokenStream;
use quote::quote;

use crate::tools::{DefaultValue, EnumValueFlatten, EnumsFlatten};

pub(crate) fn generate_enums_quote(enums: EnumsFlatten) -> Vec<TokenStream> {
    enums
        .iter()
        .map(|(k, v, default_value, extra_macros)| {
            let keys = v
                .iter()
                .map(|(key, ty, extra_macros)| {
                    let extra_macros = extra_macros
                        .iter()
                        .map(|content| {
                            quote! {
                                #[#content]
                            }
                        })
                        .collect::<Vec<_>>();

                    match ty {
                        EnumValueFlatten::Empty => quote! {
                            #(#extra_macros)*
                            #key,
                        },
                        EnumValueFlatten::Tuple(v) => quote! {
                            #(#extra_macros)*
                            #key(#(#v),*),
                        },
                        EnumValueFlatten::Struct(v) => {
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
                                        #key: #ty,
                                    }
                                })
                                .collect::<Vec<_>>();

                            quote! {
                                #(#extra_macros)*
                                #key {
                                    #( #keys )*
                                },
                            }
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

            quote! {
                #[derive(Debug, Clone)]
                #derive_macros
                #attr_macros
                pub enum #k {
                    #( #keys )*
                }

                #default_value
            }
        })
        .collect::<Vec<_>>()
}
