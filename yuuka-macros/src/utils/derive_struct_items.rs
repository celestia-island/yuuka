use std::collections::HashMap;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    token, Expr, Ident, Token, TypePath,
};

use super::{
    merge_enums, merge_structs, DefaultValue, DeriveEnum, DeriveStruct, Enums, StructMembers,
    StructName, StructType, Structs,
};

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

                let ident = if bracket_level_content.peek(Token![enum]) {
                    // sth: [enum Ident { ... }],
                    // sth: [enum { ... }],
                    let content: DeriveEnum = bracket_level_content.parse()?;
                    merge_structs(&content.sub_structs, &mut sub_structs);
                    merge_enums(&content.sub_enums, &mut sub_enums);

                    content.ident
                } else {
                    // sth: [Ident { ... }],
                    // sth: [{ ... }],
                    let content: DeriveStruct = bracket_level_content.parse()?;
                    merge_structs(&content.sub_structs, &mut sub_structs);
                    merge_enums(&content.sub_enums, &mut sub_enums);

                    content.ident
                };

                own_struct.insert(
                    key,
                    (
                        match ident {
                            StructName::Named(ident) => StructType::InlineVector(ident),
                            StructName::Unnamed(v) => StructType::UnnamedInlineVector(v),
                        },
                        {
                            if input.peek(Token![=]) {
                                // sth: [...] = ...,
                                input.parse::<Token![=]>()?;

                                let bracket_level_content;
                                bracketed!(bracket_level_content in input);
                                let mut default_value = vec![];

                                while !bracket_level_content.is_empty() {
                                    default_value.push(bracket_level_content.parse::<Expr>()?);

                                    if bracket_level_content.peek(Token![,]) {
                                        bracket_level_content.parse::<Token![,]>()?;
                                    }
                                }

                                DefaultValue::Array(default_value)
                            } else {
                                DefaultValue::None
                            }
                        },
                    ),
                );
            } else if input.peek(Token![enum]) {
                // sth: enum Ident { ... },
                // sth: enum { ... },
                let content: DeriveEnum = input.parse()?;
                merge_structs(&content.sub_structs, &mut sub_structs);
                merge_enums(&content.sub_enums, &mut sub_enums);

                own_struct.insert(
                    key.clone(),
                    (
                        {
                            match content.ident {
                                StructName::Named(ident) => StructType::Inline(ident),
                                StructName::Unnamed(v) => StructType::UnnamedInline(v),
                            }
                        },
                        {
                            if input.peek(Token![=]) {
                                input.parse::<Token![=]>()?;
                                let default_value = input.parse::<Expr>()?;
                                DefaultValue::Single(default_value)
                            } else {
                                DefaultValue::None
                            }
                        },
                    ),
                );
            } else if input.peek(token::Brace) || input.peek2(token::Brace) {
                // sth: Ident { ... },
                // sth: { ... },
                let content: DeriveStruct = input.parse()?;
                merge_structs(&content.sub_structs, &mut sub_structs);
                merge_enums(&content.sub_enums, &mut sub_enums);

                own_struct.insert(
                    key.clone(),
                    (
                        {
                            match content.ident {
                                StructName::Named(ident) => StructType::Inline(ident),
                                StructName::Unnamed(v) => StructType::UnnamedInline(v),
                            }
                        },
                        DefaultValue::None,
                    ),
                );
            } else {
                // sth: TypePath,
                let ty: TypePath = input.parse()?;

                own_struct.insert(
                    key,
                    (StructType::Static(ty), {
                        if input.peek(Token![=]) {
                            input.parse::<Token![=]>()?;
                            let default_value = input.parse::<Expr>()?;

                            DefaultValue::Single(default_value)
                        } else {
                            DefaultValue::None
                        }
                    }),
                );
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
