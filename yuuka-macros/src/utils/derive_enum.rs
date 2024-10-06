use syn::{
    braced,
    parse::{Parse, ParseStream},
    Expr, Ident, Token,
};

use crate::utils::{append_prefix_to_enums, append_prefix_to_structs};

use super::{DeriveEnumItems, Enums, StructName, Structs};

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
            StructName::Unnamed(vec![])
        };
        let content;
        braced!(content in input);
        let content: DeriveEnumItems = content.parse()?;

        let structs = append_prefix_to_structs(
            ident.to_ident().map_err(|err| {
                syn::Error::new(input.span(), format!("Invalid struct name: {}", err))
            })?,
            content.sub_structs,
        );
        let mut enums = append_prefix_to_enums(
            ident.to_ident().map_err(|err| {
                syn::Error::new(input.span(), format!("Invalid enum name: {}", err))
            })?,
            content.sub_enums,
        );

        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let default_value: Expr = input.parse()?;

            enums.insert(ident.clone(), (content.items, Some(default_value)));
        } else {
            enums.insert(ident.clone(), (content.items, None));
        }

        Ok(DeriveEnum {
            ident,
            sub_structs: structs,
            sub_enums: enums,
        })
    }
}
