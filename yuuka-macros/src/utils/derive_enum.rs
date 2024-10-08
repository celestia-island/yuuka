use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, Token,
};

use super::{DeriveEnumItems, EnumMembers, StructName};

#[derive(Debug, Clone, PartialEq)]
pub struct DeriveEnum {
    pub ident: StructName,
    pub items: EnumMembers,
}

impl Parse for DeriveEnum {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![enum]>()?;
        let ident: StructName = if input.peek(Ident) {
            StructName::Named(input.parse()?)
        } else {
            StructName::Unnamed
        };
        let content;
        braced!(content in input);
        let content: DeriveEnumItems = content.parse()?;

        Ok(DeriveEnum {
            ident,
            items: content.items,
        })
    }
}
