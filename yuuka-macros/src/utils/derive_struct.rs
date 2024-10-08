use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, Token,
};

use super::{DeriveStructItems, StructMembers, StructName};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeriveStructVisibility {
    Public,
    PublicOnCrate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeriveStruct {
    pub visibility: DeriveStructVisibility,
    pub ident: StructName,
    pub items: StructMembers,
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
            StructName::Unnamed
        };
        let content;
        braced!(content in input);
        let content: DeriveStructItems = content.parse()?;

        Ok(DeriveStruct {
            visibility,
            ident,
            items: content.items,
        })
    }
}
