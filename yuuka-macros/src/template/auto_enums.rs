use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::tools::{EnumValueFlatten, EnumsFlatten};

pub(crate) fn generate_enums_auto_macros(enums: EnumsFlatten) -> Vec<TokenStream> {
    enums
        .iter()
        .map(|(ident, v, _default_value, _extra_macros)| {
            let rules = v
                .iter()
                .map(|(name, ty, _)| match ty {
                    EnumValueFlatten::Empty => {
                        quote! {}
                    }
                    EnumValueFlatten::Struct(items) => {
                        let list = items
                            .iter()
                            .map(|(key, ty, _default_value, _extra_macros)| {
                                quote! {
                                    (#name #key { $($val:tt)+ }) => {
                                        ::yuuka::auto!(#ty { $($val)+ })
                                    };
                                }
                            })
                            .collect::<Vec<_>>();
                        quote! {
                            #(#list)*
                        }
                    }
                    EnumValueFlatten::Tuple(items) => {
                        let list = items
                            .iter()
                            .enumerate()
                            .map(|(i, ty)| {
                                let i = syn::Index::from(i);
                                quote! {
                                    (#name #i $($val:tt)+) => {
                                        ::yuuka::auto!(#ty::$($val)+)
                                    };

                                    (#name #i { $($val:tt)+ }) => {
                                        ::yuuka::auto!(#ty { $($val)+ })
                                    };
                                }
                            })
                            .collect::<Vec<_>>();
                        quote! {
                            #(#list)*
                        }
                    }
                })
                .collect::<Vec<_>>();
            let rules = quote! {
                #(#rules)*
            };

            let ident = Ident::new(format!("__auto_{}", ident).as_str(), ident.span());
            quote! {
                #[doc(hidden)]
                macro_rules! #ident {
                    () => {};

                    #rules

                    ($name:ident $key:ident $val:expr) => {
                        $val
                    };
                    ($name:ident $val:expr) => {
                        $val
                    };
                }
            }
        })
        .collect::<Vec<_>>()
}
