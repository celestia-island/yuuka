use proc_macro::TokenStream;
use quote::quote;
use std::{cell::RefCell, rc::Rc};
use syn::{parse_macro_input, Ident};

mod template;
mod tools;
mod utils;

use template::{
    generate_enums_auto_macros, generate_enums_quote, generate_structs_auto_macros,
    generate_structs_quote,
};
use tools::{
    auto_macros::AutoMacrosType, AutoMacros, DeriveBox, DeriveEnum, DeriveStruct, DeriveVisibility,
    StructName,
};
use utils::flatten;

#[proc_macro]
pub fn derive_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveStruct);

    let is_public = input.visibility == DeriveVisibility::Public;
    let macro_visibility = input.extra_macros.macros_visibility;
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
        DeriveBox::Struct(Box::new(input.clone())),
    )
    .expect("Failed to flatten");

    let structs_auto_macros = generate_structs_auto_macros(structs.clone(), macro_visibility);
    let enums_auto_macros = generate_enums_auto_macros(enums.clone(), macro_visibility);

    let structs = generate_structs_quote(structs);
    let enums = generate_enums_quote(enums);

    let ret = if is_public {
        quote! {
            #[macro_use]
            #[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
            pub mod #mod_ident {
                use super::*;

                #( #structs )*
                #( #enums )*

                #( #structs_auto_macros )*
                #( #enums_auto_macros )*
            }

            pub use #mod_ident::*;
        }
    } else {
        quote! {
            #[macro_use]
            #[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
            pub(crate) mod #mod_ident {
                use super::*;

                #( #structs )*
                #( #enums )*

                #( #structs_auto_macros )*
                #( #enums_auto_macros )*
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
    let macro_visibility = input.extra_macros.macros_visibility;
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
        DeriveBox::Enum(Box::new(input.clone())),
    )
    .expect("Failed to flatten");

    let structs_auto_macros = generate_structs_auto_macros(structs.clone(), macro_visibility);
    let enums_auto_macros = generate_enums_auto_macros(enums.clone(), macro_visibility);

    let structs = generate_structs_quote(structs);
    let enums = generate_enums_quote(enums);

    let ret = if is_public {
        quote! {
            #[macro_use]
            #[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
            pub mod #mod_ident {
                use super::*;

                #( #structs )*
                #( #enums )*

                #( #structs_auto_macros )*
                #( #enums_auto_macros )*
            }

            pub use #mod_ident::*;
        }
    } else {
        quote! {
            #[macro_use]
            #[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
            pub(crate) mod #mod_ident {
                use super::*;

                #( #structs )*
                #( #enums )*

                #( #structs_auto_macros )*
                #( #enums_auto_macros )*
            }

            pub(crate) use #mod_ident::*;
        }
    };

    ret.into()
}

#[proc_macro]
pub fn auto(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AutoMacros);
    let ident = input.ident.clone();
    let body = input.body;

    let macro_ident = Ident::new(&format!("__auto_{}", input.ident), input.ident.span());
    match body {
        AutoMacrosType::Struct {
            items,
            expand_exprs,
        } => {
            let list = items
                .iter()
                .map(|(key, value)| {
                    quote! {
                        #key: #macro_ident!(#key #value)
                    }
                })
                .collect::<Vec<_>>();

            if let Some(expand_exprs) = expand_exprs {
                quote! {
                    #ident {
                        #( #list ),*,
                        ..#expand_exprs
                    }
                }
                .into()
            } else {
                quote! {
                    #ident {
                        #( #list ),*
                    }
                }
                .into()
            }
        }

        AutoMacrosType::EnumEmpty { key } => quote! {
            #ident::#key
        }
        .into(),
        AutoMacrosType::EnumStruct {
            key,
            items,
            expand_exprs,
        } => {
            let list = items
                .iter()
                .map(|(item_key, value)| {
                    quote! {
                        #item_key: #macro_ident!(#key #item_key #value)
                    }
                })
                .collect::<Vec<_>>();

            if let Some(expand_exprs) = expand_exprs {
                quote! {
                    #ident::#key {
                        #( #list ),*,
                        ..#expand_exprs
                    }
                }
                .into()
            } else {
                quote! {
                    #ident::#key {
                        #( #list ),*
                    }
                }
                .into()
            }
        }
        AutoMacrosType::EnumTuple { key, items } => {
            if items.len() == 1 {
                let first_item = items.first().expect("Failed to get first item");
                quote! {
                    #ident::#key(#macro_ident!(#key #first_item))
                }
                .into()
            } else {
                let list = items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| {
                        quote! {
                            #macro_ident!(#key #index #item)
                        }
                    })
                    .collect::<Vec<_>>();

                quote! {
                    #ident::#key(#( #list ),*)
                }
                .into()
            }
        }
        AutoMacrosType::EnumSinglePath { key, next_key } => quote! {
            #ident::#key(#macro_ident!(#key 0 #next_key))
        }
        .into(),

        AutoMacrosType::Value { items } => {
            if items.len() == 1 {
                let first_item = items.first().expect("Failed to get first item");
                quote! {
                    #first_item
                }
                .into()
            } else {
                let list = items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| {
                        quote! {
                            #macro_ident!(#index #item)
                        }
                    })
                    .collect::<Vec<_>>();

                quote! {
                    (#( #list ),*)
                }
                .into()
            }
        }
    }
}
