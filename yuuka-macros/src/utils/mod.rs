use anyhow::Result;
use syn::{parse_quote, Expr, Ident, TypePath};

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

pub(crate) type StructMembers = Vec<(Ident, StructType, DefaultValue)>;
pub(crate) type EnumMembers = Vec<(Ident, EnumValue)>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum EnumValueFlatten {
    Empty,
    Tuple(Vec<TypePath>),
    Struct(Vec<(Ident, TypePath, DefaultValue)>),
}
pub(crate) type StructsFlatten = Vec<(Ident, Vec<(Ident, TypePath, DefaultValue)>)>;
pub(crate) type EnumsFlatten = Vec<(Ident, Vec<(Ident, EnumValueFlatten)>, DefaultValue)>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DeriveBox {
    Struct(DeriveStruct),
    Enum(DeriveEnum),
}
pub(crate) fn flatten(
    unique_id_count: &mut usize,
    parent: DeriveBox,
) -> Result<(StructsFlatten, EnumsFlatten)> {
    match parent {
        DeriveBox::Struct(parent) => {
            let mut structs = vec![];
            let mut enums = vec![];

            let mut items = vec![];
            for (key, ty, default_value) in parent.items.iter() {
                match ty {
                    StructType::Static(v) => {
                        items.push((key.clone(), v.clone(), default_value.clone()));
                    }
                    StructType::InlineStruct(v) => {
                        let (sub_structs, sub_enums) =
                            flatten(unique_id_count, DeriveBox::Struct(v.clone()))?;
                        structs.extend(sub_structs);
                        enums.extend(sub_enums);

                        let ty = if let StructName::Named(ident) = v.ident.clone() {
                            ident
                        } else {
                            let ident = Ident::new(
                                &format!("_{}_anonymous", unique_id_count),
                                proc_macro2::Span::call_site(),
                            );
                            *unique_id_count += 1;
                            ident
                        };
                        items.push((key.clone(), parse_quote! { #ty }, default_value.clone()));
                    }
                    StructType::InlineStructVector(v) => {
                        let (sub_structs, sub_enums) =
                            flatten(unique_id_count, DeriveBox::Struct(v.clone()))?;
                        structs.extend(sub_structs);
                        enums.extend(sub_enums);

                        let ty = if let StructName::Named(ident) = v.ident.clone() {
                            ident
                        } else {
                            let ident = Ident::new(
                                &format!("_{}_anonymous", unique_id_count),
                                proc_macro2::Span::call_site(),
                            );
                            *unique_id_count += 1;
                            ident
                        };
                        items.push((
                            key.clone(),
                            parse_quote! { Vec<#ty> },
                            default_value.clone(),
                        ));
                    }
                    StructType::InlineEnum(v) => {
                        let (sub_structs, sub_enums) =
                            flatten(unique_id_count, DeriveBox::Enum(v.clone()))?;
                        structs.extend(sub_structs);
                        enums.extend(sub_enums);

                        let ty = if let StructName::Named(ident) = v.ident.clone() {
                            ident
                        } else {
                            let ident = Ident::new(
                                &format!("_{}_anonymous", unique_id_count),
                                proc_macro2::Span::call_site(),
                            );
                            *unique_id_count += 1;
                            ident
                        };
                        items.push((key.clone(), parse_quote! { #ty }, default_value.clone()));
                    }
                    StructType::InlineEnumVector(v) => {
                        let (sub_structs, sub_enums) =
                            flatten(unique_id_count, DeriveBox::Enum(v.clone()))?;
                        structs.extend(sub_structs);
                        enums.extend(sub_enums);

                        let ty = if let StructName::Named(ident) = v.ident.clone() {
                            ident
                        } else {
                            let ident = Ident::new(
                                &format!("_{}_anonymous", unique_id_count),
                                proc_macro2::Span::call_site(),
                            );
                            *unique_id_count += 1;
                            ident
                        };
                        items.push((
                            key.clone(),
                            parse_quote! { Vec<#ty> },
                            default_value.clone(),
                        ));
                    }
                }
            }

            let ident = if let StructName::Named(ident) = parent.ident.clone() {
                ident
            } else {
                let ident = Ident::new(
                    &format!("_{}_anonymous", unique_id_count),
                    proc_macro2::Span::call_site(),
                );
                *unique_id_count += 1;
                ident
            };
            structs.push((ident, items));

            Ok((structs, enums))
        }
        DeriveBox::Enum(parent) => {
            let mut structs = vec![];
            let mut enums = vec![];

            let mut items = vec![];
            for (key, value) in parent.items.iter() {
                match value {
                    EnumValue::Empty => {
                        items.push((key.clone(), EnumValueFlatten::Empty));
                    }
                    EnumValue::Tuple(v) => {
                        let mut tuple = vec![];
                        for ty in v.iter() {
                            match ty {
                                StructType::Static(v) => {
                                    tuple.push(v.clone());
                                }
                                StructType::InlineStruct(v) => {
                                    let (sub_structs, sub_enums) =
                                        flatten(unique_id_count, DeriveBox::Struct(v.clone()))?;
                                    structs.extend(sub_structs);
                                    enums.extend(sub_enums);

                                    let ty = if let StructName::Named(ident) = v.ident.clone() {
                                        ident
                                    } else {
                                        let ret = Ident::new(
                                            &format!("_{}_anonymous", unique_id_count),
                                            key.span(),
                                        );
                                        *unique_id_count += 1;
                                        ret
                                    };
                                    tuple.push(parse_quote! { #ty });
                                }
                                StructType::InlineStructVector(v) => {
                                    let (sub_structs, sub_enums) =
                                        flatten(unique_id_count, DeriveBox::Struct(v.clone()))?;
                                    structs.extend(sub_structs);
                                    enums.extend(sub_enums);

                                    let ty = if let StructName::Named(ident) = v.ident.clone() {
                                        ident
                                    } else {
                                        let ret = Ident::new(
                                            &format!("_{}_anonymous", unique_id_count),
                                            key.span(),
                                        );
                                        *unique_id_count += 1;
                                        ret
                                    };
                                    tuple.push(parse_quote! { Vec<#ty> });
                                }
                                StructType::InlineEnum(v) => {
                                    let (sub_structs, sub_enums) =
                                        flatten(unique_id_count, DeriveBox::Enum(v.clone()))?;
                                    structs.extend(sub_structs);
                                    enums.extend(sub_enums);

                                    let ty = if let StructName::Named(ident) = v.ident.clone() {
                                        ident
                                    } else {
                                        let ret = Ident::new(
                                            &format!("_{}_anonymous", unique_id_count),
                                            key.span(),
                                        );
                                        *unique_id_count += 1;
                                        ret
                                    };
                                    tuple.push(parse_quote! { #ty });
                                }
                                StructType::InlineEnumVector(v) => {
                                    let (sub_structs, sub_enums) =
                                        flatten(unique_id_count, DeriveBox::Enum(v.clone()))?;
                                    structs.extend(sub_structs);
                                    enums.extend(sub_enums);

                                    let ty = if let StructName::Named(ident) = v.ident.clone() {
                                        ident
                                    } else {
                                        let ret = Ident::new(
                                            &format!("_{}_anonymous", unique_id_count),
                                            key.span(),
                                        );
                                        *unique_id_count += 1;
                                        ret
                                    };
                                    tuple.push(parse_quote! { Vec<#ty> });
                                }
                            }
                        }
                        items.push((key.clone(), EnumValueFlatten::Tuple(tuple)));
                    }
                    EnumValue::Struct(v) => {
                        let mut sub_items = vec![];
                        for (key, ty, default_value) in v.iter() {
                            match ty {
                                StructType::Static(v) => {
                                    sub_items.push((key.clone(), v.clone(), default_value.clone()));
                                }
                                StructType::InlineStruct(v) => {
                                    let (sub_structs, sub_enums) =
                                        flatten(unique_id_count, DeriveBox::Struct(v.clone()))?;
                                    structs.extend(sub_structs);
                                    enums.extend(sub_enums);

                                    let ty = if let StructName::Named(ident) = v.ident.clone() {
                                        ident
                                    } else {
                                        let ret = Ident::new(
                                            &format!("_{}_anonymous", unique_id_count),
                                            key.span(),
                                        );
                                        *unique_id_count += 1;
                                        ret
                                    };
                                    sub_items.push((
                                        key.clone(),
                                        parse_quote! { #ty },
                                        default_value.clone(),
                                    ));
                                }
                                StructType::InlineStructVector(v) => {
                                    let (sub_structs, sub_enums) =
                                        flatten(unique_id_count, DeriveBox::Struct(v.clone()))?;
                                    structs.extend(sub_structs);
                                    enums.extend(sub_enums);

                                    let ty = if let StructName::Named(ident) = v.ident.clone() {
                                        ident
                                    } else {
                                        let ret = Ident::new(
                                            &format!("_{}_anonymous", unique_id_count),
                                            key.span(),
                                        );
                                        *unique_id_count += 1;
                                        ret
                                    };
                                    sub_items.push((
                                        key.clone(),
                                        parse_quote! { Vec<#ty> },
                                        default_value.clone(),
                                    ));
                                }
                                StructType::InlineEnum(v) => {
                                    let (sub_structs, sub_enums) =
                                        flatten(unique_id_count, DeriveBox::Enum(v.clone()))?;
                                    structs.extend(sub_structs);
                                    enums.extend(sub_enums);

                                    let ty = if let StructName::Named(ident) = v.ident.clone() {
                                        ident
                                    } else {
                                        let ret = Ident::new(
                                            &format!("_{}_anonymous", unique_id_count),
                                            key.span(),
                                        );
                                        *unique_id_count += 1;
                                        ret
                                    };
                                    sub_items.push((
                                        key.clone(),
                                        parse_quote! { #ty },
                                        default_value.clone(),
                                    ));
                                }
                                StructType::InlineEnumVector(v) => {
                                    let (sub_structs, sub_enums) =
                                        flatten(unique_id_count, DeriveBox::Enum(v.clone()))?;
                                    structs.extend(sub_structs);
                                    enums.extend(sub_enums);

                                    let ty = if let StructName::Named(ident) = v.ident.clone() {
                                        ident
                                    } else {
                                        let ret = Ident::new(
                                            &format!("_{}_anonymous", unique_id_count),
                                            key.span(),
                                        );
                                        *unique_id_count += 1;
                                        ret
                                    };
                                    sub_items.push((
                                        key.clone(),
                                        parse_quote! { Vec<#ty> },
                                        default_value.clone(),
                                    ));
                                }
                            }
                        }

                        items.push((key.clone(), EnumValueFlatten::Struct(sub_items)));
                    }
                }
            }

            let ident = if let StructName::Named(ident) = parent.ident.clone() {
                ident
            } else {
                let ident = Ident::new(
                    &format!("_{}_anonymous", unique_id_count),
                    proc_macro2::Span::call_site(),
                );
                *unique_id_count += 1;
                ident
            };
            enums.push((
                ident.clone(),
                items,
                if let Some(value) = parent.default_value {
                    DefaultValue::Single(parse_quote! { Self::#value })
                } else {
                    DefaultValue::None
                },
            ));

            Ok((structs, enums))
        }
    }
}
