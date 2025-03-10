use anyhow::Result;
use std::{cell::RefCell, rc::Rc};
use syn::parse_quote;

use crate::tools::{
    DefaultValue, DeriveBox, EnumValue, EnumValueFlatten, EnumsFlatten, ExtraMacrosFlatten,
    ExtraTypeWrapper, StructType, StructsFlatten,
};

pub(crate) fn flatten(
    root_name: String,
    unique_id_count: Rc<RefCell<usize>>,
    parent: DeriveBox,
) -> Result<(StructsFlatten, EnumsFlatten)> {
    match parent {
        DeriveBox::Struct(parent) => {
            let mut structs = vec![];
            let mut enums = vec![];

            let mut items = vec![];
            for (key, ty, extra_type_wrapper, default_value, extra_macros) in parent.items.iter() {
                match ty {
                    StructType::Static(v) => {
                        items.push((
                            key.clone(),
                            match extra_type_wrapper {
                                ExtraTypeWrapper::Option => parse_quote! { Option<#v> },
                                ExtraTypeWrapper::OptionVec => parse_quote! { Option<#v> },
                                _ => v.clone(),
                            },
                            default_value.clone(),
                            extra_macros.attr_macros.clone(),
                        ));
                    }
                    StructType::InlineStruct(v) => {
                        let v = v
                            .clone()
                            .pin_unique_id(root_name.clone(), unique_id_count.clone());
                        let v = if let Some(derive_macros) = extra_macros.derive_macros.clone() {
                            v.extend_attr_macros(derive_macros.attr_macros)
                                .extend_attr_macros_recursive(derive_macros.attr_macros_recursive)
                        } else {
                            v
                        };
                        let v = if let Some(derive_macros) =
                            parent.extra_macros.derive_macros.clone()
                        {
                            v.extend_derive_macros(derive_macros.derive_macros)
                                .extend_attr_macros_recursive(derive_macros.attr_macros_recursive)
                        } else {
                            v
                        };

                        let (sub_structs, sub_enums) = flatten(
                            root_name.clone(),
                            unique_id_count.clone(),
                            DeriveBox::Struct(Box::new(v.clone())),
                        )?;

                        structs.extend(sub_structs);
                        enums.extend(sub_enums);

                        let ty = v.ident.to_ident()?;
                        items.push((
                            key.clone(),
                            match extra_type_wrapper {
                                ExtraTypeWrapper::Default => parse_quote! { #ty },
                                ExtraTypeWrapper::Vec => parse_quote! { Vec<#ty> },
                                ExtraTypeWrapper::Option => parse_quote! { Option<#ty> },
                                ExtraTypeWrapper::OptionVec => parse_quote! { Option<Vec<#ty>> },
                            },
                            default_value.clone(),
                            extra_macros.attr_macros.clone(),
                        ));
                    }
                    StructType::InlineEnum(v) => {
                        let v = v
                            .clone()
                            .pin_unique_id(root_name.clone(), unique_id_count.clone());
                        let v = if let Some(derive_macros) = extra_macros.derive_macros.clone() {
                            v.extend_attr_macros(derive_macros.attr_macros)
                                .extend_attr_macros_recursive(derive_macros.attr_macros_recursive)
                        } else {
                            v
                        };
                        let v = if let Some(derive_macros) =
                            parent.extra_macros.derive_macros.clone()
                        {
                            v.extend_derive_macros(derive_macros.derive_macros)
                                .extend_attr_macros_recursive(derive_macros.attr_macros_recursive)
                        } else {
                            v
                        };

                        let (sub_structs, sub_enums) = flatten(
                            root_name.clone(),
                            unique_id_count.clone(),
                            DeriveBox::Enum(Box::new(v.clone())),
                        )?;

                        structs.extend(sub_structs);
                        enums.extend(sub_enums);

                        let ty = v.ident.to_ident()?;
                        items.push((
                            key.clone(),
                            match extra_type_wrapper {
                                ExtraTypeWrapper::Default => parse_quote! { #ty },
                                ExtraTypeWrapper::Vec => parse_quote! { Vec<#ty> },
                                ExtraTypeWrapper::Option => parse_quote! { Option<#ty> },
                                ExtraTypeWrapper::OptionVec => parse_quote! { Option<Vec<#ty>> },
                            },
                            default_value.clone(),
                            extra_macros.attr_macros.clone(),
                        ));
                    }
                }
            }

            let ty = parent.ident.to_ident()?;
            structs.push((
                ty,
                items,
                ExtraMacrosFlatten {
                    derive_macros: parent
                        .extra_macros
                        .derive_macros
                        .clone()
                        .map(|derive_macros| derive_macros.derive_macros)
                        .unwrap_or_default(),
                    attr_macros: parent
                        .extra_macros
                        .derive_macros
                        .map(|derive_macros| {
                            [
                                derive_macros.attr_macros.clone(),
                                derive_macros.attr_macros_recursive.clone(),
                            ]
                            .concat()
                        })
                        .unwrap_or_default(),
                },
            ));

            Ok((structs, enums))
        }
        DeriveBox::Enum(parent) => {
            let mut structs = vec![];
            let mut enums = vec![];

            let mut items = vec![];
            for (key, value, extra_macros) in parent.items.iter() {
                match value {
                    EnumValue::Empty => {
                        items.push((
                            key.clone(),
                            EnumValueFlatten::Empty,
                            extra_macros.attr_macros.clone(),
                        ));
                    }
                    EnumValue::Tuple(v) => {
                        let mut tuple = vec![];
                        for (ty, extra_type_wrapper) in v.iter() {
                            match ty {
                                StructType::Static(v) => {
                                    tuple.push(match extra_type_wrapper {
                                        ExtraTypeWrapper::Option => parse_quote! { Option<#v> },
                                        ExtraTypeWrapper::OptionVec => parse_quote! { Option<#v> },
                                        _ => v.clone(),
                                    });
                                }
                                StructType::InlineStruct(v) => {
                                    let v = v
                                        .clone()
                                        .pin_unique_id(root_name.clone(), unique_id_count.clone());
                                    let v = if let Some(derive_macros) =
                                        extra_macros.derive_macros.clone()
                                    {
                                        v.extend_attr_macros(derive_macros.attr_macros)
                                            .extend_attr_macros_recursive(
                                                derive_macros.attr_macros_recursive,
                                            )
                                    } else {
                                        v
                                    };
                                    let v = if let Some(derive_macros) =
                                        parent.extra_macros.derive_macros.clone()
                                    {
                                        v.extend_derive_macros(derive_macros.derive_macros)
                                            .extend_attr_macros_recursive(
                                                derive_macros.attr_macros_recursive,
                                            )
                                    } else {
                                        v
                                    };

                                    let (sub_structs, sub_enums) = flatten(
                                        root_name.clone(),
                                        unique_id_count.clone(),
                                        DeriveBox::Struct(Box::new(v.clone())),
                                    )?;

                                    structs.extend(sub_structs);
                                    enums.extend(sub_enums);

                                    let ty = v.ident.to_ident()?;
                                    tuple.push(match extra_type_wrapper {
                                        ExtraTypeWrapper::Default => parse_quote! { #ty },
                                        ExtraTypeWrapper::Vec => parse_quote! { Vec<#ty> },
                                        ExtraTypeWrapper::Option => parse_quote! { Option<#ty> },
                                        ExtraTypeWrapper::OptionVec => {
                                            parse_quote! { Option<Vec<#ty>> }
                                        }
                                    });
                                }
                                StructType::InlineEnum(v) => {
                                    let v = v
                                        .clone()
                                        .pin_unique_id(root_name.clone(), unique_id_count.clone());
                                    let v = if let Some(derive_macros) =
                                        extra_macros.derive_macros.clone()
                                    {
                                        v.extend_attr_macros(derive_macros.attr_macros)
                                            .extend_attr_macros_recursive(
                                                derive_macros.attr_macros_recursive,
                                            )
                                    } else {
                                        v
                                    };
                                    let v = if let Some(derive_macros) =
                                        parent.extra_macros.derive_macros.clone()
                                    {
                                        v.extend_derive_macros(derive_macros.derive_macros)
                                            .extend_attr_macros_recursive(
                                                derive_macros.attr_macros_recursive,
                                            )
                                    } else {
                                        v
                                    };

                                    let (sub_structs, sub_enums) = flatten(
                                        root_name.clone(),
                                        unique_id_count.clone(),
                                        DeriveBox::Enum(Box::new(v.clone())),
                                    )?;

                                    structs.extend(sub_structs);
                                    enums.extend(sub_enums);

                                    let ty = v.ident.to_ident()?;
                                    tuple.push(match extra_type_wrapper {
                                        ExtraTypeWrapper::Default => parse_quote! { #ty },
                                        ExtraTypeWrapper::Vec => parse_quote! { Vec<#ty> },
                                        ExtraTypeWrapper::Option => parse_quote! { Option<#ty> },
                                        ExtraTypeWrapper::OptionVec => {
                                            parse_quote! { Option<Vec<#ty>> }
                                        }
                                    });
                                }
                            }
                        }
                        items.push((
                            key.clone(),
                            EnumValueFlatten::Tuple(tuple),
                            extra_macros.attr_macros.clone(),
                        ));
                    }
                    EnumValue::Struct(v) => {
                        let mut sub_items = vec![];
                        for (key, ty, extra_type_wrapper, default_value, extra_macros) in v.iter() {
                            match ty {
                                StructType::Static(v) => {
                                    sub_items.push((
                                        key.clone(),
                                        match extra_type_wrapper {
                                            ExtraTypeWrapper::Option => parse_quote! { Option<#v> },
                                            ExtraTypeWrapper::OptionVec => {
                                                parse_quote! { Option<#v> }
                                            }
                                            _ => v.clone(),
                                        },
                                        default_value.clone(),
                                        extra_macros.attr_macros.clone(),
                                    ));
                                }
                                StructType::InlineStruct(v) => {
                                    let v = v
                                        .clone()
                                        .pin_unique_id(root_name.clone(), unique_id_count.clone());
                                    let v = if let Some(derive_macros) =
                                        extra_macros.derive_macros.clone()
                                    {
                                        v.extend_attr_macros(derive_macros.attr_macros)
                                            .extend_attr_macros_recursive(
                                                derive_macros.attr_macros_recursive,
                                            )
                                    } else {
                                        v
                                    };
                                    let v = if let Some(derive_macros) =
                                        parent.extra_macros.derive_macros.clone()
                                    {
                                        v.extend_derive_macros(derive_macros.derive_macros)
                                            .extend_attr_macros_recursive(
                                                derive_macros.attr_macros_recursive,
                                            )
                                    } else {
                                        v
                                    };

                                    let (sub_structs, sub_enums) = flatten(
                                        root_name.clone(),
                                        unique_id_count.clone(),
                                        DeriveBox::Struct(Box::new(v.clone())),
                                    )?;

                                    structs.extend(sub_structs);
                                    enums.extend(sub_enums);

                                    let ty = v.ident.to_ident()?;
                                    sub_items.push((
                                        key.clone(),
                                        match extra_type_wrapper {
                                            ExtraTypeWrapper::Default => parse_quote! { #ty },
                                            ExtraTypeWrapper::Vec => parse_quote! { Vec<#ty> },
                                            ExtraTypeWrapper::Option => {
                                                parse_quote! { Option<#ty> }
                                            }
                                            ExtraTypeWrapper::OptionVec => {
                                                parse_quote! { Option<Vec<#ty>> }
                                            }
                                        },
                                        default_value.clone(),
                                        extra_macros.attr_macros.clone(),
                                    ));
                                }
                                StructType::InlineEnum(v) => {
                                    let v = v
                                        .clone()
                                        .pin_unique_id(root_name.clone(), unique_id_count.clone());
                                    let v = if let Some(derive_macros) =
                                        extra_macros.derive_macros.clone()
                                    {
                                        v.extend_attr_macros(derive_macros.attr_macros)
                                            .extend_attr_macros_recursive(
                                                derive_macros.attr_macros_recursive,
                                            )
                                    } else {
                                        v
                                    };
                                    let v = if let Some(derive_macros) =
                                        parent.extra_macros.derive_macros.clone()
                                    {
                                        v.extend_derive_macros(derive_macros.derive_macros)
                                            .extend_attr_macros_recursive(
                                                derive_macros.attr_macros_recursive,
                                            )
                                    } else {
                                        v
                                    };

                                    let (sub_structs, sub_enums) = flatten(
                                        root_name.clone(),
                                        unique_id_count.clone(),
                                        DeriveBox::Enum(Box::new(v.clone())),
                                    )?;

                                    structs.extend(sub_structs);
                                    enums.extend(sub_enums);

                                    let ty = v.ident.to_ident()?;
                                    sub_items.push((
                                        key.clone(),
                                        match extra_type_wrapper {
                                            ExtraTypeWrapper::Default => parse_quote! { #ty },
                                            ExtraTypeWrapper::Vec => parse_quote! { Vec<#ty> },
                                            ExtraTypeWrapper::Option => {
                                                parse_quote! { Option<#ty> }
                                            }
                                            ExtraTypeWrapper::OptionVec => {
                                                parse_quote! { Option<Vec<#ty>> }
                                            }
                                        },
                                        default_value.clone(),
                                        extra_macros.attr_macros.clone(),
                                    ));
                                }
                            }
                        }

                        items.push((
                            key.clone(),
                            EnumValueFlatten::Struct(sub_items),
                            extra_macros.attr_macros.clone(),
                        ));
                    }
                }
            }

            let ty = parent.ident.to_ident()?;
            enums.push((
                ty,
                items,
                if let Some(value) = parent.default_value {
                    DefaultValue::Single(parse_quote! { Self::#value })
                } else {
                    DefaultValue::None
                },
                ExtraMacrosFlatten {
                    derive_macros: parent
                        .extra_macros
                        .derive_macros
                        .clone()
                        .map(|derive_macros| derive_macros.derive_macros)
                        .unwrap_or_default(),
                    attr_macros: parent
                        .extra_macros
                        .derive_macros
                        .map(|derive_macros| {
                            [
                                derive_macros.attr_macros.clone(),
                                derive_macros.attr_macros_recursive.clone(),
                            ]
                            .concat()
                        })
                        .unwrap_or_default(),
                },
            ));

            Ok((structs, enums))
        }
    }
}
