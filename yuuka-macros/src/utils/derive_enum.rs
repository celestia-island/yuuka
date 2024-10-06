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
            StructName::Unnamed(Default::default())
        };
        let content;
        braced!(content in input);
        let content: DeriveEnumItems = content.parse()?;

        let structs = append_prefix_to_structs(ident.to_ident()?, content.sub_structs);
        let mut enums = append_prefix_to_enums(ident.to_ident()?, content.sub_enums);

        enums.insert(
            ident.clone(),
            (content.items, {
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                    let default_value: Expr = input.parse()?;

                    Some(default_value)
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
