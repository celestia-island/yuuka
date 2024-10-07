use anyhow::Result;
use std::collections::HashMap;

use quote::ToTokens;
use syn::{Expr, Ident, TypePath};

pub(crate) mod derive_enum;
pub(crate) mod derive_enum_items;
pub(crate) mod derive_struct_items;

pub mod derive_struct;

pub(crate) use derive_enum::DeriveEnum;
pub(crate) use derive_enum_items::DeriveEnumItems;
pub(crate) use derive_struct_items::DeriveStructItems;

pub use derive_struct::DeriveStruct;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DefaultValue {
    None,
    Single(Expr),
    Array(Vec<Expr>),
}

#[derive(Debug, Clone)]
pub enum StructParentPath {
    Path(Vec<(Ident, usize)>),
    Empty(proc_macro2::Span),
}

impl PartialEq for StructParentPath {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StructParentPath::Path(v1), StructParentPath::Path(v2)) => v1 == v2,
            (StructParentPath::Empty(v1), StructParentPath::Empty(v2)) => {
                v1.start() == v2.start() && v1.end() == v2.end()
            }
            _ => false,
        }
    }
}
impl Eq for StructParentPath {}

impl std::hash::Hash for StructParentPath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            StructParentPath::Path(v) => v.hash(state),
            StructParentPath::Empty(v) => format!(
                "{},line:{:?},column:{:?}",
                v.source_text().unwrap_or_default(),
                v.start(),
                v.end()
            )
            .hash(&mut std::hash::DefaultHasher::new()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum StructName {
    Named(Ident),
    Unnamed(StructParentPath),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum StructType {
    Static(TypePath),
    Inline(Ident),
    InlineVector(Ident),
    UnnamedInline(StructParentPath),
    UnnamedInlineVector(StructParentPath),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum EnumValue {
    Empty,
    Tuple(Vec<StructType>),
    Struct(StructMembers),
}

impl StructParentPath {
    pub(crate) fn unshift(&self, prefix: Ident, index: usize) -> StructParentPath {
        match self {
            StructParentPath::Path(v) => {
                let mut v = v.clone();
                v.insert(0, (prefix, index));
                StructParentPath::Path(v)
            }
            StructParentPath::Empty(_) => StructParentPath::Path(vec![(prefix, index)]),
        }
    }

    pub(crate) fn to_type_path(&self) -> Result<TypePath, syn::Error> {
        Ok(match self {
            StructParentPath::Path(v) => {
                let v = v
                    .iter()
                    .map(|(ident, index)| format!("{}_{}", ident, index))
                    .collect::<Vec<_>>()
                    .join("_");
                syn::parse_str(&format!("{}_anonymous", v))?
            }
            StructParentPath::Empty(v) => {
                return Err(syn::Error::new(
                    *v,
                    "Empty parent path is not allowed in this context",
                ))
            }
        })
    }
}

impl StructType {
    pub(crate) fn to_type_path(&self) -> Result<TypePath, syn::Error> {
        Ok(match self {
            StructType::Static(v) => v.clone(),
            StructType::Inline(v) => syn::parse_str(&format!("{}", v))?,
            StructType::InlineVector(v) => syn::parse_str(&format!("Vec<{}>", v))?,
            StructType::UnnamedInline(v) => v.to_type_path()?,
            StructType::UnnamedInlineVector(v) => syn::parse_str(&format!(
                "Vec<{}>",
                v.to_type_path()?.to_token_stream().to_string()
            ))?,
        })
    }

    pub(crate) fn push_prefix(&self, prefix: Ident, index: usize) -> StructType {
        match self {
            StructType::Static(v) => StructType::Static(v.clone()),
            StructType::Inline(v) => StructType::Inline(v.clone()),
            StructType::InlineVector(v) => StructType::InlineVector(v.clone()),
            StructType::UnnamedInline(v) => StructType::UnnamedInline(v.unshift(prefix, index)),
            StructType::UnnamedInlineVector(v) => {
                StructType::UnnamedInlineVector(v.unshift(prefix, index))
            }
        }
    }
}

impl StructName {
    pub(crate) fn to_ident(&self) -> Result<Ident, syn::Error> {
        Ok(match self {
            StructName::Named(v) => v.clone(),
            StructName::Unnamed(v) => syn::parse_str(&format!(
                "{}",
                v.to_type_path()?.to_token_stream().to_string()
            ))?,
        })
    }

    pub(crate) fn push_prefix(&self, prefix: Ident, index: usize) -> StructName {
        match self {
            StructName::Named(v) => StructName::Named(v.clone()),
            StructName::Unnamed(v) => StructName::Unnamed(v.unshift(prefix, index)),
        }
    }
}

pub(crate) type Structs = HashMap<StructName, StructMembers>;
pub(crate) type StructMembers = Vec<(Ident, StructType, DefaultValue)>;
pub(crate) type Enums = HashMap<StructName, (EnumMembers, Option<Expr>)>;
pub(crate) type EnumMembers = Vec<(Ident, EnumValue)>;

pub(crate) fn merge_structs(source: &Structs, target: &mut Structs) {
    for (k, v) in source.iter() {
        if target.contains_key(&k) {
            panic!("Duplicate key `{:?}`", k);
        }
        target.insert(k.clone(), v.clone());
    }
}

pub(crate) fn merge_enums(source: &Enums, target: &mut Enums) {
    for (k, v) in source.iter() {
        if target.contains_key(&k) {
            panic!("Duplicate key `{:?}`", k);
        }
        target.insert(k.clone(), v.clone());
    }
}

pub(crate) fn append_prefix_to_structs(prefix: Ident, structs: Structs) -> Structs {
    let mut new_structs = HashMap::new();

    for (index, (k, v)) in structs.iter().enumerate() {
        let k = k.push_prefix(prefix.clone(), index);
        let v = v
            .iter()
            .map(|(key, ty, default_value)| {
                (
                    key.clone(),
                    ty.push_prefix(prefix.clone(), index),
                    default_value.clone(),
                )
            })
            .collect();
        new_structs.insert(k, v);
    }

    new_structs
}

pub(crate) fn append_prefix_to_enums(prefix: Ident, enums: Enums) -> Enums {
    let mut new_enums = HashMap::new();

    for (index, (k, v)) in enums.iter().enumerate() {
        let k = k.push_prefix(prefix.clone(), index);
        let v = (
            v.0.iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        match value {
                            EnumValue::Empty => EnumValue::Empty,
                            EnumValue::Tuple(v) => EnumValue::Tuple(
                                v.iter()
                                    .map(|v| v.push_prefix(prefix.clone(), index))
                                    .collect(),
                            ),
                            EnumValue::Struct(v) => EnumValue::Struct(
                                v.iter()
                                    .map(|(key, ty, default_value)| {
                                        (
                                            key.clone(),
                                            ty.push_prefix(prefix.clone(), index),
                                            default_value.clone(),
                                        )
                                    })
                                    .collect(),
                            ),
                        },
                    )
                })
                .collect(),
            v.1.clone(),
        );
        new_enums.insert(k, v.clone());
    }

    new_enums
}
