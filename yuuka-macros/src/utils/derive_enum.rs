use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, Token,
};

use super::{DeriveEnumItems, Enums, Structs};

#[derive(Debug, Clone)]
pub struct DeriveEnum {
    pub ident: Option<Ident>,

    pub sub_structs: Structs,
    pub sub_enums: Enums,
}

impl Parse for DeriveEnum {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![enum]>()?;
        let ident: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let content: DeriveEnumItems = content.parse()?;

        let mut enums = content.sub_enums;
        enums.insert(ident.clone(), content.items);

        Ok(DeriveEnum {
            ident: Some(ident),
            sub_structs: content.sub_structs,
            sub_enums: enums,
        })
    }
}
