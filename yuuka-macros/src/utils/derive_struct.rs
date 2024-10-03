use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident,
};

use super::{DeriveStructItems, Enums, Structs};

#[derive(Debug, Clone)]
pub struct DeriveStruct {
    pub ident: Option<Ident>,

    pub sub_structs: Structs,
    pub sub_enums: Enums,
}

impl Parse for DeriveStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let content: DeriveStructItems = content.parse()?;

        let mut structs = content.sub_structs;
        structs.insert(ident.clone(), content.items);

        Ok(DeriveStruct {
            ident: Some(ident),
            sub_structs: structs,
            sub_enums: content.sub_enums,
        })
    }
}
