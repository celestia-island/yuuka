use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, Token,
};

use crate::utils::{append_prefix_to_enums, append_prefix_to_structs};

use super::{DeriveStructItems, Enums, StructName, Structs};

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
            StructName::Unnamed(vec![])
        };
        let content;
        braced!(content in input);
        let content: DeriveStructItems = content.parse()?;

        let mut structs = append_prefix_to_structs(
            ident.to_ident().map_err(|err| {
                syn::Error::new(input.span(), format!("Invalid struct name: {}", err))
            })?,
            content.sub_structs,
        );
        let enums = append_prefix_to_enums(
            ident.to_ident().map_err(|err| {
                syn::Error::new(input.span(), format!("Invalid struct name: {}", err))
            })?,
            content.sub_enums,
        );

        structs.insert(ident.clone(), content.items);

        dbg!(structs.clone());
        Ok(DeriveStruct {
            visibility,
            ident,
            sub_structs: structs,
            sub_enums: enums,
        })
    }
}
