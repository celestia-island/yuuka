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

/// Generate nested structs from a concise DSL-like syntax.
///
/// Supports inline struct definitions, arrays, enums, Option-wrapped fields,
/// custom derives, visibility control, and default values. See the crate-level
/// documentation for full syntax and examples.
#[proc_macro]
pub fn derive_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveStruct);

    let is_public = input.visibility == DeriveVisibility::Public;
    let macro_visibility = input.extra_macros.macros_visibility;
    let root_ident = match input.ident.clone() {
        StructName::Named(v) => v,
        StructName::Unnamed(_) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "unnamed root struct is not supported; provide a name for the struct",
            )
            .to_compile_error()
            .into();
        }
    };
    let mod_ident = syn::Ident::new(&format!("__{}", root_ident), root_ident.span());
    let (structs, enums) = match flatten(
        root_ident.to_string(),
        Rc::new(RefCell::new(0)),
        DeriveBox::Struct(Box::new(input.clone())),
    ) {
        Ok(v) => v,
        Err(e) => {
            return syn::Error::new(root_ident.span(), e.to_string())
                .to_compile_error()
                .into();
        }
    };

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

/// Generate enums (and associated structs) from a concise DSL-like syntax.
///
/// Supports unit variants, tuple variants, and struct variants with inline
/// member definitions. Default values, custom derives, and visibility are
/// controlled via the same syntax as `derive_struct`.
#[proc_macro]
pub fn derive_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveEnum);

    let is_public = input.visibility == DeriveVisibility::Public;
    let macro_visibility = input.extra_macros.macros_visibility;
    let root_ident = match input.ident.clone() {
        StructName::Named(v) => v,
        StructName::Unnamed(_) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "unnamed root enum is not supported; provide a name for the enum",
            )
            .to_compile_error()
            .into();
        }
    };
    let mod_ident = syn::Ident::new(&format!("__{}", root_ident), root_ident.span());
    let (structs, enums) = match flatten(
        root_ident.to_string(),
        Rc::new(RefCell::new(0)),
        DeriveBox::Enum(Box::new(input.clone())),
    ) {
        Ok(v) => v,
        Err(e) => {
            return syn::Error::new(root_ident.span(), e.to_string())
                .to_compile_error()
                .into();
        }
    };

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

/// Construct an instance of a type generated by `derive_struct` or
/// `derive_enum` using a minimal value-only syntax.
///
/// Each field value is automatically routed to the correct generated
/// `__auto_*` helper macro so that nested inline types are constructed
/// transparently.
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
            if let [first_item] = items.as_slice() {
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
            if let [first_item] = items.as_slice() {
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
