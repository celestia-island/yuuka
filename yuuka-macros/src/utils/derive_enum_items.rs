use std::collections::HashMap;
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    token, Ident, Token, TypePath,
};

use super::{
    merge_enums, merge_structs, DeriveEnum, DeriveStruct, DeriveStructItems, EnumMembers,
    EnumValue, Enums, Structs,
};

#[derive(Debug, Clone)]
pub struct DeriveEnumItems {
    pub items: EnumMembers,

    pub sub_structs: Structs,
    pub sub_enums: Enums,
}

impl Parse for DeriveEnumItems {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut sub_structs: Structs = HashMap::new();
        let mut sub_enums: Enums = HashMap::new();

        let mut own_enum: EnumMembers = HashMap::new();

        while !input.is_empty() {
            let key = input.parse::<Ident>()?;

            let value = if input.peek(token::Brace) {
                // Ident { ... },
                let sub_content;
                braced!(sub_content in input);
                let content: DeriveStructItems = sub_content.parse()?;
                merge_structs(&content.sub_structs, &mut sub_structs);
                merge_enums(&content.sub_enums, &mut sub_enums);

                EnumValue::Struct(content.items)
            } else if input.peek(token::Paren) {
                // Ident(...),
                let sub_content;
                parenthesized!(sub_content in input);
                let mut tuple: Vec<TypePath> = Vec::new();

                while !sub_content.is_empty() {
                    if sub_content.peek(token::Bracket) {
                        // Ident([...], ...),
                        let bracket_level_content;
                        bracketed!(bracket_level_content in sub_content);

                        if bracket_level_content.peek(Token![enum]) {
                            // Ident([enum { ... }], ...),
                            let content: DeriveEnum = bracket_level_content.parse()?;
                            merge_structs(&content.sub_structs, &mut sub_structs);
                            merge_enums(&content.sub_enums, &mut sub_enums);

                            tuple.push(syn::parse_str::<TypePath>(&format!(
                                "Vec<{}>",
                                content.ident.to_ident()?,
                            ))?);
                        } else {
                            // Ident([Ident { ... }], ...),
                            let content: DeriveStruct = bracket_level_content.parse()?;
                            merge_structs(&content.sub_structs, &mut sub_structs);
                            merge_enums(&content.sub_enums, &mut sub_enums);

                            tuple.push(syn::parse_str::<TypePath>(&format!(
                                "Vec<{}>",
                                content.ident.to_ident()?,
                            ))?);
                        }
                    } else if sub_content.peek(Token![enum]) {
                        // Ident(enum Ident { ... }, ...),
                        let content: DeriveEnum = sub_content.parse()?;
                        merge_structs(&content.sub_structs, &mut sub_structs);
                        merge_enums(&content.sub_enums, &mut sub_enums);

                        tuple.push(syn::parse_str::<TypePath>(
                            &content.ident.to_ident()?.to_string(),
                        )?);
                    } else if sub_content.peek2(token::Brace) {
                        // Ident(Ident { ... }, ...),
                        let content: DeriveStruct = sub_content.parse()?;
                        merge_structs(&content.sub_structs, &mut sub_structs);
                        merge_enums(&content.sub_enums, &mut sub_enums);

                        tuple.push(syn::parse_str::<TypePath>(
                            &content.ident.to_ident()?.to_string(),
                        )?);
                    } else {
                        // Ident (Ident, ...),
                        let ty: TypePath = sub_content.parse()?;
                        tuple.push(ty);
                    }

                    if sub_content.peek(Token![,]) {
                        sub_content.parse::<Token![,]>()?;
                    }
                }

                EnumValue::Tuple(tuple)
            } else {
                // Ident,
                EnumValue::Empty
            };

            own_enum.insert(key, value);

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(DeriveEnumItems {
            items: own_enum,
            sub_structs,
            sub_enums,
        })
    }
}
