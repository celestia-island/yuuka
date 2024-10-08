use anyhow::Result;
use syn::{Expr, Ident, TypePath};

pub(crate) mod derive_enum;
pub(crate) mod derive_enum_items;
pub(crate) mod derive_struct;
pub(crate) mod derive_struct_items;

pub(crate) use derive_enum::DeriveEnum;
pub(crate) use derive_enum_items::DeriveEnumItems;
pub(crate) use derive_struct::DeriveStruct;
pub(crate) use derive_struct_items::DeriveStructItems;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DefaultValue {
    None,
    Single(Expr),
    Array(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StructName {
    Named(Ident),
    Unnamed,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StructType {
    Static(TypePath),
    InlineStruct(DeriveStruct),
    InlineStructVector(DeriveStruct),
    InlineEnum(DeriveEnum),
    InlineEnumVector(DeriveEnum),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum EnumValue {
    Empty,
    Tuple(Vec<StructType>),
    Struct(StructMembers),
}

pub(crate) type Structs = Vec<(StructName, StructMembers)>;
pub(crate) type StructMembers = Vec<(Ident, StructType, DefaultValue)>;
pub(crate) type Enums = Vec<(StructName, EnumMembers, Option<Expr>)>;
pub(crate) type EnumMembers = Vec<(Ident, EnumValue)>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum EnumValueFlatten {
    Empty,
    Tuple(Vec<TypePath>),
    Struct(StructsFlatten),
}
pub(crate) type StructsFlatten = Vec<(Ident, TypePath, DefaultValue)>;
pub(crate) type EnumsFlatten = Vec<(Ident, Vec<(Ident, EnumValue)>, DefaultValue)>;

pub(crate) fn flatten(root: DeriveStruct) -> Result<(StructsFlatten, EnumsFlatten)> {
    todo!()
}
