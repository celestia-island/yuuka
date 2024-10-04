use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, Token,
};

use super::{DeriveStructItems, Enums, Structs};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeriveStructVisibility {
    Public,
    PublicOnCrate,
}

#[derive(Debug, Clone)]
pub struct DeriveStruct {
    pub visibility: DeriveStructVisibility,
    pub ident: Option<Ident>,

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

        let ident: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let content: DeriveStructItems = content.parse()?;

        let mut structs = content.sub_structs;
        structs.insert(ident.clone(), content.items);

        Ok(DeriveStruct {
            visibility,
            ident: Some(ident),
            sub_structs: structs,
            sub_enums: content.sub_enums,
        })
    }
}
