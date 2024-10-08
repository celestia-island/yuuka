use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_quote, token, Expr, Ident, Token, TypePath,
};

use super::{DefaultValue, DeriveEnum, DeriveStruct, StructMembers, StructName, StructType};

#[derive(Debug, Clone)]
pub struct DeriveStructItems {
    pub items: StructMembers,
}

impl Parse for DeriveStructItems {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut own_struct: StructMembers = Vec::new();

        while !input.is_empty() {
            let key = input.parse::<Ident>()?;
            input.parse::<Token![:]>()?;

            if input.peek(token::Bracket) {
                // sth: [...],

                let bracket_level_content;
                bracketed!(bracket_level_content in input);

                if bracket_level_content.peek(Token![enum]) {
                    // sth: [enum Ident { ... }],
                    // sth: [enum { ... }],
                    let content: DeriveEnum = bracket_level_content.parse()?;

                    own_struct.push((
                        key,
                        match content.ident {
                            StructName::Named(ident) => {
                                StructType::Static(parse_quote! { Vec<#ident> })
                            }
                            StructName::Unnamed => StructType::InlineEnumVector(content),
                        },
                        {
                            if input.peek(Token![=]) {
                                input.parse::<Token![=]>()?;
                                let default_value = input.parse::<Expr>()?;

                                if input.peek(token::Brace) {
                                    // sth: [enum Ident { ... } = { ... }],
                                    // sth: [enum { ... } = { ... }],

                                    let sub_content;
                                    bracketed!(sub_content in input);

                                    let mut default_values = Vec::new();
                                    while !sub_content.is_empty() {
                                        default_values.push(sub_content.parse::<Expr>()?);
                                        if sub_content.peek(Token![,]) {
                                            sub_content.parse::<Token![,]>()?;
                                        }
                                    }

                                    DefaultValue::Array(default_values)
                                } else {
                                    // sth: [enum Ident { ... } = ...],
                                    // sth: [enum { ... } = ...],
                                    DefaultValue::Single(default_value)
                                }
                            } else {
                                DefaultValue::None
                            }
                        },
                    ));
                } else {
                    // sth: [Ident { ... }],
                    // sth: [{ ... }],
                    let content: DeriveStruct = bracket_level_content.parse()?;

                    own_struct.push((
                        key,
                        match content.ident {
                            StructName::Named(ident) => {
                                StructType::Static(parse_quote! { Vec<#ident> })
                            }
                            StructName::Unnamed => StructType::InlineStructVector(content),
                        },
                        {
                            if input.peek(Token![=]) {
                                input.parse::<Token![=]>()?;
                                let default_value = input.parse::<Expr>()?;

                                if input.peek(token::Brace) {
                                    // sth: [Ident { ... } = { ... }],
                                    // sth: [{ ... } = { ... }],

                                    let sub_content;
                                    bracketed!(sub_content in input);

                                    let mut default_values = Vec::new();
                                    while !sub_content.is_empty() {
                                        default_values.push(sub_content.parse::<Expr>()?);
                                        if sub_content.peek(Token![,]) {
                                            sub_content.parse::<Token![,]>()?;
                                        }
                                    }

                                    DefaultValue::Array(default_values)
                                } else {
                                    // sth: [Ident { ... } = ...],
                                    // sth: [{ ... } = ...],
                                    DefaultValue::Single(default_value)
                                }
                            } else {
                                DefaultValue::None
                            }
                        },
                    ));
                };
            } else if input.peek(Token![enum]) {
                // sth: enum Ident { ... },
                // sth: enum { ... },
                let content: DeriveEnum = input.parse()?;

                own_struct.push((
                    key.clone(),
                    {
                        match content.ident {
                            StructName::Named(ident) => StructType::Static(parse_quote! { #ident }),
                            StructName::Unnamed => StructType::InlineEnum(content),
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
                ));
            } else if input.peek(token::Brace) || input.peek2(token::Brace) {
                // sth: Ident { ... },
                // sth: { ... },
                let content: DeriveStruct = input.parse()?;

                own_struct.push((
                    key.clone(),
                    {
                        match content.ident {
                            StructName::Named(ident) => StructType::Static(parse_quote! { #ident }),
                            StructName::Unnamed => StructType::InlineStruct(content),
                        }
                    },
                    DefaultValue::None,
                ));
            } else {
                // sth: TypePath,
                let ty: TypePath = input.parse()?;

                own_struct.push((key, StructType::Static(ty), {
                    if input.peek(Token![=]) {
                        input.parse::<Token![=]>()?;
                        let default_value = input.parse::<Expr>()?;

                        DefaultValue::Single(default_value)
                    } else {
                        DefaultValue::None
                    }
                }));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(DeriveStructItems { items: own_struct })
    }
}
