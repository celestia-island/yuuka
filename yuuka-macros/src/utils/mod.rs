use anyhow::Result;
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
pub(crate) enum EnumValue {
    Empty,
    Tuple(Vec<TypePath>),
    Struct(StructMembers),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DefaultValue {
    None,
    Single(Expr),
    Array(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum StructName {
    Named(Ident),
    Unnamed(Vec<Ident>),
}

impl StructName {
    pub(crate) fn to_ident(&self) -> Result<Ident, syn::Error> {
        Ok(match self {
            StructName::Named(v) => v.clone(),
            StructName::Unnamed(v) => syn::parse_str(
                &v.iter()
                    .fold(String::new(), |acc, ident| format!("{}_{}", acc, ident)),
            )?,
        })
    }
}

pub(crate) type Structs = HashMap<StructName, StructMembers>;
pub(crate) type StructMembers = HashMap<Ident, (TypePath, DefaultValue)>;
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

    for (k, v) in structs.iter() {
        let k = match k {
            StructName::Named(v) => StructName::Named(v.clone()),
            StructName::Unnamed(v) => {
                let v = v
                    .iter()
                    .map(|v| Ident::new(&format!("{}_{}", prefix, v), v.span()))
                    .collect();
                StructName::Unnamed(v)
            }
        };
        new_structs.insert(k, v.clone());
    }

    new_structs
}

pub(crate) fn append_prefix_to_enums(prefix: Ident, enums: Enums) -> Enums {
    let mut new_enums = HashMap::new();

    for (k, v) in enums.iter() {
        let k = match k {
            StructName::Named(v) => StructName::Named(v.clone()),
            StructName::Unnamed(v) => {
                let v = v
                    .iter()
                    .map(|v| Ident::new(&format!("{}_{}", prefix, v), v.span()))
                    .collect();
                StructName::Unnamed(v)
            }
        };
        new_enums.insert(k, v.clone());
    }

    new_enums
}
