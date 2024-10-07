use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, Token,
};

use super::{
    append_prefix_to_enums, append_prefix_to_structs, DeriveEnumItems, EnumValue, Enums,
    StructName, StructParentPath, StructType, Structs,
};

#[derive(Debug, Clone)]
pub struct DeriveEnum {
    pub ident: StructName,

    pub sub_structs: Structs,
    pub sub_enums: Enums,
}

impl Parse for DeriveEnum {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![enum]>()?;
        let ident: StructName = if input.peek(Ident) {
            StructName::Named(input.parse()?)
        } else {
            StructName::Unnamed(StructParentPath::Empty(input.span()))
        };
        let content;
        braced!(content in input);
        let content: DeriveEnumItems = content.parse()?;

        let structs = append_prefix_to_structs(ident.to_ident()?, content.sub_structs);
        let mut enums = append_prefix_to_enums(ident.to_ident()?, content.sub_enums);

        let mut items = content.items;
        for (_k, v) in items.iter_mut() {
            if let EnumValue::Tuple(v) = v {
                for (index, v) in v.iter_mut().enumerate() {
                    if let StructType::UnnamedInline(v) = v {
                        *v = v.unshift(ident.to_ident()?, index);
                    } else if let StructType::UnnamedInlineVector(v) = v {
                        *v = v.unshift(ident.to_ident()?, index);
                    }
                }
            } else if let EnumValue::Struct(v) = v {
                for (index, (_key, ty, _default_value)) in v.iter_mut().enumerate() {
                    if let StructType::UnnamedInline(ty) = ty {
                        *ty = ty.unshift(ident.to_ident()?, index);
                    } else if let StructType::UnnamedInlineVector(ty) = ty {
                        *ty = ty.unshift(ident.to_ident()?, index);
                    }
                }
            }
        }

        enums.insert(
            ident.clone(),
            (items, {
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                    Some(input.parse()?)
                } else {
                    None
                }
            }),
        );

        Ok(DeriveEnum {
            ident,
            sub_structs: structs,
            sub_enums: enums,
        })
    }
}
