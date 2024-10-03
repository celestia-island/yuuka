use std::collections::HashMap;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    token, Ident, Token, TypePath,
};

use crate::utils::DeriveStruct;

use super::{merge_enums, merge_structs, DeriveEnum, Enums, StructMembers, Structs};

#[derive(Debug, Clone)]
pub struct DeriveStructItems {
    pub items: StructMembers,

    pub sub_structs: Structs,
    pub sub_enums: Enums,
}

impl Parse for DeriveStructItems {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut sub_structs: Structs = HashMap::new();
        let mut sub_enums: Enums = HashMap::new();

        let mut own_struct: StructMembers = HashMap::new();

        while !input.is_empty() {
            let key = input.parse::<Ident>()?;
            input.parse::<Token![:]>()?;

            if input.peek(token::Bracket) {
                // sth: [...],

                let bracket_level_content;
                bracketed!(bracket_level_content in input);

                if bracket_level_content.peek(Token![enum]) {
                    // sth: [enum Ident { ... }],
                    let content: DeriveEnum = bracket_level_content.parse()?;
                    merge_structs(&content.sub_structs, &mut sub_structs);
                    merge_enums(&content.sub_enums, &mut sub_enums);

                    own_struct.insert(
                        key,
                        syn::parse_str::<TypePath>(&format!(
                            "Vec<{}>",
                            content
                                .ident
                                .ok_or(syn::Error::new(
                                    bracket_level_content.span(),
                                    "Anonymous struct is not support yet."
                                ))?
                                .to_string()
                        ))?,
                    );
                } else {
                    // sth: [Ident { ... }],
                    let content: DeriveStruct = bracket_level_content.parse()?;
                    merge_structs(&content.sub_structs, &mut sub_structs);
                    merge_enums(&content.sub_enums, &mut sub_enums);

                    own_struct.insert(
                        key,
                        syn::parse_str::<TypePath>(&format!(
                            "Vec<{}>",
                            content.ident.ok_or(syn::Error::new(
                                bracket_level_content.span(),
                                "Anonymous struct is not support yet."
                            ))?
                        ))?,
                    );
                }
            } else if input.peek(Token![enum]) {
                // sth: enum Ident { ... },
                let content: DeriveEnum = input.parse()?;
                merge_structs(&content.sub_structs, &mut sub_structs);
                merge_enums(&content.sub_enums, &mut sub_enums);

                own_struct.insert(
                    key,
                    syn::parse_str::<TypePath>(
                        &content
                            .ident
                            .ok_or(syn::Error::new(
                                input.span(),
                                "Anonymous struct is not support yet.",
                            ))?
                            .to_string(),
                    )?,
                );
            } else if input.peek2(token::Brace) {
                // sth: Ident { ... },
                let content: DeriveStruct = input.parse()?;
                merge_structs(&content.sub_structs, &mut sub_structs);
                merge_enums(&content.sub_enums, &mut sub_enums);

                own_struct.insert(
                    key,
                    syn::parse_str::<TypePath>(
                        &content
                            .ident
                            .ok_or(syn::Error::new(
                                input.span(),
                                "Anonymous struct is not support yet.",
                            ))?
                            .to_string(),
                    )?,
                );
            } else {
                // sth: TypePath,
                let ty = input.parse()?;
                own_struct.insert(key, ty);
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(DeriveStructItems {
            items: own_struct,

            sub_structs,
            sub_enums,
        })
    }
}