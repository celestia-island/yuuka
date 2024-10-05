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

#[derive(Debug, Clone)]
pub enum EnumValue {
    Empty,
    Tuple(Vec<TypePath>),
    Struct(StructMembers),
}

pub(crate) type Structs = HashMap<Ident, StructMembers>;
pub(crate) type StructMembers = HashMap<Ident, (TypePath, Option<Expr>)>;
pub(crate) type Enums = HashMap<Ident, (EnumMembers, Option<Expr>)>;
pub(crate) type EnumMembers = HashMap<Ident, EnumValue>;

pub(crate) fn merge_structs(source: &Structs, target: &mut Structs) {
    for (k, v) in source.iter() {
        if target.contains_key(&k) {
            panic!("duplicate key `{}`", k);
        }
        target.insert(k.clone(), v.clone());
    }
}

pub(crate) fn merge_enums(source: &Enums, target: &mut Enums) {
    for (k, v) in source.iter() {
        if target.contains_key(&k) {
            panic!("duplicate key `{}`", k);
        }
        target.insert(k.clone(), v.clone());
    }
}
