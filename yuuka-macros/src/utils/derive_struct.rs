use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, Token,
};

use super::{
    append_prefix_to_enums, append_prefix_to_structs, DeriveStructItems, Enums, StructName,
    StructParentPath, StructType, Structs,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeriveStructVisibility {
    Public,
    PublicOnCrate,
}

#[derive(Debug, Clone)]
pub struct DeriveStruct {
    pub visibility: DeriveStructVisibility,
    pub ident: StructName,

    pub sub_structs: Structs,
    pub sub_enums: Enums,
}

impl Parse for DeriveStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let visibility = if input.peek(Token![pub]) {
            input.parse::<Token![pub]>()?;
            DeriveStructVisibility::Public
        } else {
            DeriveStructVisibility::PublicOnCrate
        };

        let ident: StructName = if input.peek(Ident) {
            StructName::Named(input.parse()?)
        } else {
            StructName::Unnamed(StructParentPath::Empty(input.span()))
        };
        let content;
        braced!(content in input);
        let content: DeriveStructItems = content.parse()?;

        let mut structs = append_prefix_to_structs(ident.to_ident()?, content.sub_structs);
        let enums = append_prefix_to_enums(ident.to_ident()?, content.sub_enums);

        let mut items = content.items;
        for (index, (_key, ty, _default_value)) in items.iter_mut().enumerate() {
            if let StructType::UnnamedInline(ty) = ty {
                *ty = ty.unshift(ident.to_ident()?, index);
            } else if let StructType::UnnamedInlineVector(ty) = ty {
                *ty = ty.unshift(ident.to_ident()?, index);
            }
        }

        structs.insert(ident.clone(), items);

        Ok(DeriveStruct {
            visibility,
            ident,
            sub_structs: structs,
            sub_enums: enums,
        })
    }
}
