use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    token, Expr, Ident, Token, TypePath,
};

use crate::tools::ExtraTypeWrapper;

use super::{DefaultValue, DeriveEnum, DeriveStruct, ExtraMacros, StructMembers, StructType};

#[derive(Debug, Clone)]
pub struct DeriveStructItems {
    pub items: StructMembers,
}

impl Parse for DeriveStructItems {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut own_struct: StructMembers = Vec::new();

        while !input.is_empty() {
            let extra_macros = if input.peek(Token![#]) {
                input.parse::<ExtraMacros>()?
            } else {
                Default::default()
            };

            let key = input.parse::<Ident>()?;
            let optional = if input.peek(Token![?]) {
                input.parse::<Token![?]>()?;
                true
            } else {
                false
            };
            input.parse::<Token![:]>()?;

            if input.peek(token::Bracket) {
                // sth: [...],

                let bracket_level_content;
                bracketed!(bracket_level_content in input);

                if bracket_level_content.peek(Token![enum]) {
                    // sth: [enum Ident { ... }],
                    // sth: [enum { ... }],
                    let content: DeriveEnum = bracket_level_content.parse()?;
                    let content = if let Some(derive_macros) = extra_macros.derive_macros.clone() {
                        content
                            .extend_derive_macros(derive_macros.derive_macros)
                            .extend_attr_macros_recursive(derive_macros.attr_macros_recursive)
                    } else {
                        content
                    };

                    own_struct.push((
                        key,
                        StructType::InlineEnum(content),
                        if optional {
                            ExtraTypeWrapper::OptionVec
                        } else {
                            ExtraTypeWrapper::Vec
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
                                    DefaultValue::Single(Box::new(default_value))
                                }
                            } else {
                                DefaultValue::None
                            }
                        },
                        extra_macros,
                    ));
                } else {
                    // sth: [Ident { ... }],
                    // sth: [{ ... }],
                    let content: DeriveStruct = bracket_level_content.parse()?;
                    let content = if let Some(derive_macros) = extra_macros.derive_macros.clone() {
                        content
                            .extend_derive_macros(derive_macros.derive_macros)
                            .extend_attr_macros_recursive(derive_macros.attr_macros_recursive)
                    } else {
                        content
                    };

                    own_struct.push((
                        key,
                        StructType::InlineStruct(content),
                        if optional {
                            ExtraTypeWrapper::OptionVec
                        } else {
                            ExtraTypeWrapper::Vec
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
                                    DefaultValue::Single(Box::new(default_value))
                                }
                            } else {
                                DefaultValue::None
                            }
                        },
                        extra_macros,
                    ));
                };
            } else if input.peek(Token![enum]) {
                // sth: enum Ident { ... },
                // sth: enum { ... },
                let content: DeriveEnum = input.parse()?;
                let content = if let Some(derive_macros) = extra_macros.derive_macros.clone() {
                    content
                        .extend_derive_macros(derive_macros.derive_macros)
                        .extend_attr_macros_recursive(derive_macros.attr_macros_recursive)
                } else {
                    content
                };

                own_struct.push((
                    key.clone(),
                    StructType::InlineEnum(content),
                    if optional {
                        ExtraTypeWrapper::Option
                    } else {
                        ExtraTypeWrapper::Default
                    },
                    {
                        if input.peek(Token![=]) {
                            input.parse::<Token![=]>()?;
                            let default_value = input.parse::<Expr>()?;
                            DefaultValue::Single(Box::new(default_value))
                        } else {
                            DefaultValue::None
                        }
                    },
                    extra_macros,
                ));
            } else if input.peek(token::Brace) || input.peek2(token::Brace) {
                // sth: Ident { ... },
                // sth: { ... },
                let content: DeriveStruct = input.parse()?;
                let content = if let Some(derive_macros) = extra_macros.derive_macros.clone() {
                    content
                        .extend_derive_macros(derive_macros.derive_macros)
                        .extend_attr_macros_recursive(derive_macros.attr_macros_recursive)
                } else {
                    content
                };

                own_struct.push((
                    key.clone(),
                    StructType::InlineStruct(content),
                    if optional {
                        ExtraTypeWrapper::Option
                    } else {
                        ExtraTypeWrapper::Default
                    },
                    DefaultValue::None,
                    extra_macros,
                ));
            } else {
                // sth: TypePath,
                let ty: TypePath = input.parse()?;

                own_struct.push((
                    key,
                    StructType::Static(ty),
                    if optional {
                        ExtraTypeWrapper::Option
                    } else {
                        ExtraTypeWrapper::Default
                    },
                    {
                        if input.peek(Token![=]) {
                            input.parse::<Token![=]>()?;
                            let default_value = input.parse::<Expr>()?;

                            DefaultValue::Single(Box::new(default_value))
                        } else {
                            DefaultValue::None
                        }
                    },
                    extra_macros,
                ));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(DeriveStructItems { items: own_struct })
    }
}
