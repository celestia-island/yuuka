use anyhow::Result;
use quote::ToTokens;
use std::collections::HashMap;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct StructParentPath {
    pub parents: Vec<(Ident, usize)>,
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
        let mut parents = self.parents.clone();
        parents.insert(0, (prefix, index));
        StructParentPath { parents }
    }

    pub(crate) fn to_type_path(&self) -> Result<TypePath, syn::Error> {
        syn::parse_str(&format!(
            "{}_anonymous",
            self.parents
                .iter()
                .fold(String::new(), |acc, (ident, index)| format!(
                    "{}_{}_{}",
                    acc, ident, index
                )),
        ))
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
pub(crate) type StructMembers = HashMap<Ident, (StructType, DefaultValue)>;
pub(crate) type Enums = HashMap<StructName, (EnumMembers, Option<Expr>)>;
pub(crate) type EnumMembers = HashMap<Ident, EnumValue>;

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
            .map(|(k, (v, default_value))| {
                let k = k.clone();
                (
                    k,
                    (v.push_prefix(prefix.clone(), index), default_value.clone()),
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
                .map(|(k, v)| {
                    let k = k.clone();
                    (
                        k,
                        match v {
                            EnumValue::Empty => EnumValue::Empty,
                            EnumValue::Tuple(v) => EnumValue::Tuple(
                                v.iter()
                                    .map(|v| v.push_prefix(prefix.clone(), index))
                                    .collect(),
                            ),
                            EnumValue::Struct(v) => EnumValue::Struct(
                                v.iter()
                                    .map(|(k, (v, default_value))| {
                                        let k = k.clone();
                                        (
                                            k,
                                            (
                                                v.push_prefix(prefix.clone(), index),
                                                default_value.clone(),
                                            ),
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
