use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Ident, TypePath};

pub(crate) mod auto_macros;
pub(crate) mod derive_enum;
pub(crate) mod derive_enum_items;
pub(crate) mod derive_macros_token;
pub(crate) mod derive_struct;
pub(crate) mod derive_struct_items;

pub(crate) use auto_macros::AutoMacros;
pub(crate) use derive_enum::DeriveEnum;
pub(crate) use derive_enum_items::DeriveEnumItems;
pub(crate) use derive_macros_token::ExtraMacros;
pub(crate) use derive_struct::DeriveStruct;
pub(crate) use derive_struct_items::DeriveStructItems;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum DeriveVisibility {
    Public,
    #[default]
    PublicOnCrate,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum DeriveAutoMacrosVisibility {
    Public,
    #[default]
    PublicOnCrate,
}

#[derive(Clone)]
pub(crate) enum DefaultValue {
    None,
    Single(Box<Expr>),
    Array(Vec<Expr>),
}

impl PartialEq for DefaultValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DefaultValue::None, DefaultValue::None) => true,
            (DefaultValue::Single(a), DefaultValue::Single(b)) => {
                quote!(#a).to_string() == quote!(#b).to_string()
            }
            (DefaultValue::Array(a), DefaultValue::Array(b)) => {
                let a: Vec<String> = a.iter().map(|e| quote!(#e).to_string()).collect();
                let b: Vec<String> = b.iter().map(|e| quote!(#e).to_string()).collect();
                a == b
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum StructName {
    Named(Ident),
    Unnamed(Option<(String, usize)>),
}

impl StructName {
    pub(crate) fn to_ident(&self) -> Result<Ident, syn::Error> {
        Ok(match self {
            StructName::Named(v) => v.clone(),
            StructName::Unnamed(Some((root_name, v))) => Ident::new(
                &format!("_{}_{}_anonymous", root_name, v),
                proc_macro2::Span::call_site(),
            ),
            _ => {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "Unnamed struct is not supported",
                ))
            }
        })
    }

    pub(crate) fn pin_unique_id(&self, root_name: String, unique_id: usize) -> Self {
        match self {
            StructName::Named(v) => StructName::Named(v.clone()),
            StructName::Unnamed(v) => {
                if let Some(v) = v {
                    StructName::Unnamed(Some(v.clone()))
                } else {
                    StructName::Unnamed(Some((root_name, unique_id)))
                }
            }
        }
    }
}

#[derive(Clone)]
pub(crate) enum StructType {
    Static(TypePath),
    InlineStruct(Box<DeriveStruct>),
    InlineEnum(Box<DeriveEnum>),
}

#[derive(Clone)]
pub(crate) enum EnumValue {
    Empty,
    Tuple(Vec<(StructType, ExtraTypeWrapper)>),
    Struct(StructMembers),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ExtraTypeWrapper {
    Default,
    Vec,
    Option,
    OptionVec,
}

#[derive(Clone)]
pub(crate) struct ExtraMacrosFlatten {
    pub(crate) derive_macros: Vec<TypePath>,
    pub(crate) attr_macros: Vec<TokenStream>,
}

pub(crate) type StructMembers = Vec<(
    Ident,
    StructType,
    ExtraTypeWrapper,
    DefaultValue,
    ExtraMacros,
)>;
pub(crate) type EnumMembers = Vec<(Ident, EnumValue, ExtraMacros)>;

#[derive(Clone)]
pub(crate) enum EnumValueFlatten {
    Empty,
    Tuple(Vec<TypePath>),
    Struct(Vec<(Ident, TypePath, DefaultValue, Vec<TokenStream>)>),
}
pub(crate) type StructsFlatten = Vec<(
    Ident,
    Vec<(Ident, TypePath, DefaultValue, Vec<TokenStream>)>,
    ExtraMacrosFlatten,
)>;
pub(crate) type EnumsFlatten = Vec<(
    Ident,
    Vec<(Ident, EnumValueFlatten, Vec<TokenStream>)>,
    DefaultValue,
    ExtraMacrosFlatten,
)>;

#[derive(Clone)]
pub(crate) enum DeriveBox {
    Struct(Box<DeriveStruct>),
    Enum(Box<DeriveEnum>),
}
