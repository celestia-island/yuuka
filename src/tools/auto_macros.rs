use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    token, Expr, Ident, Token,
};

#[derive(Debug, Clone)]
pub enum AutoMacrosType {
    Struct {
        items: Vec<(Ident, TokenStream)>,
        expand_exprs: Option<Expr>,
    },
    EnumEmpty {
        key: Ident,
    },
    EnumStruct {
        key: Ident,
        items: Vec<(Ident, TokenStream)>,
        expand_exprs: Option<Expr>,
    },
    EnumTuple {
        key: Ident,
        items: Vec<TokenStream>,
    },
    EnumSinglePath {
        key: Ident,
        next_key: TokenStream,
    },
    Value {
        items: Vec<Expr>,
    },
}

#[derive(Debug, Clone)]
pub struct AutoMacros {
    pub ident: Ident,
    pub body: AutoMacrosType,
}

impl Parse for AutoMacros {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;

        if input.peek(token::Brace) {
            // Str { key: ..., ... }
            let content;
            braced!(content in input);

            let mut items = vec![];
            let mut expand_exprs = None;

            while !content.is_empty() {
                if content.peek(Token![..]) {
                    if expand_exprs.is_some() {
                        return Err(content.error("Expand expression is already set"));
                    }

                    content.parse::<Token![..]>()?;
                    let value: Expr = content.parse()?;
                    expand_exprs = Some(value);

                    if !content.is_empty() {
                        return Err(content.error("Expand expression should be the last"));
                    }
                    break;
                }

                let key: Ident = content.parse()?;
                content.parse::<Token![:]>()?;

                if content.peek(token::Brace) {
                    let inner_content;
                    braced!(inner_content in content);
                    let inner_content: TokenStream = inner_content.parse()?;
                    items.push((
                        key,
                        quote! {
                            { #inner_content }
                        },
                    ));
                } else if content.peek(token::Bracket) {
                    let inner_content;
                    bracketed!(inner_content in content);
                    let inner_content: TokenStream = inner_content.parse()?;
                    items.push((
                        key,
                        quote! {
                             [ #inner_content ]
                        },
                    ));
                } else if content.peek(token::Paren) {
                    let inner_content;
                    parenthesized!(inner_content in content);
                    let inner_content: TokenStream = inner_content.parse()?;
                    items.push((
                        key,
                        quote! {
                            ( #inner_content )
                        },
                    ));
                } else {
                    let value: Expr = content.parse()?;
                    items.push((
                        key,
                        quote! {
                            #value
                        },
                    ));
                }

                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }

            Ok(AutoMacros {
                ident,
                body: AutoMacrosType::Struct {
                    items,
                    expand_exprs,
                },
            })
        } else if input.peek(Token![::]) {
            // Sth::...

            input.parse::<Token![::]>()?;
            let key: Ident = input.parse()?;

            if input.peek(token::Brace) {
                // Sth::Sth { key: ..., ... }

                let content;
                braced!(content in input);

                let mut items = vec![];
                let mut expand_exprs = None;

                while !content.is_empty() {
                    if content.peek(Token![..]) {
                        if expand_exprs.is_some() {
                            return Err(content.error("Expand expression is already set"));
                        }

                        content.parse::<Token![..]>()?;
                        let value: Expr = content.parse()?;
                        expand_exprs = Some(value);

                        if !content.is_empty() {
                            return Err(content.error("Expand expression should be the last"));
                        }
                        break;
                    }

                    let key: Ident = content.parse()?;
                    content.parse::<Token![:]>()?;

                    if content.peek(token::Brace) {
                        let inner_content;
                        braced!(inner_content in content);
                        let inner_content: TokenStream = inner_content.parse()?;
                        items.push((
                            key,
                            quote! {
                                { #inner_content }
                            },
                        ));
                    } else if content.peek(token::Bracket) {
                        let inner_content;
                        bracketed!(inner_content in content);
                        let inner_content: TokenStream = inner_content.parse()?;
                        items.push((
                            key,
                            quote! {
                                 [ #inner_content ]
                            },
                        ));
                    } else if content.peek(token::Paren) {
                        let inner_content;
                        parenthesized!(inner_content in content);
                        let inner_content: TokenStream = inner_content.parse()?;
                        items.push((
                            key,
                            quote! {
                                ( #inner_content )
                            },
                        ));
                    } else {
                        let value: Expr = content.parse()?;
                        items.push((
                            key,
                            quote! {
                                #value
                            },
                        ));
                    }

                    if content.peek(Token![,]) {
                        content.parse::<Token![,]>()?;
                    }
                }

                Ok(AutoMacros {
                    ident,
                    body: AutoMacrosType::EnumStruct {
                        key,
                        items,
                        expand_exprs,
                    },
                })
            } else if input.peek(token::Paren) {
                // Sth::Sth(key, ...)

                let content;
                parenthesized!(content in input);

                let mut items = vec![];
                while !content.is_empty() {
                    let value: Expr = content.parse()?;
                    items.push(quote! {
                        #value
                    });

                    if content.peek(Token![,]) {
                        content.parse::<Token![,]>()?;
                    }
                }

                Ok(AutoMacros {
                    ident,
                    body: AutoMacrosType::EnumTuple { key, items },
                })
            } else if input.peek(Token![::]) {
                // Sth::Sth::Sth

                input.parse::<Token![::]>()?;
                let next_key: TokenStream = input.parse()?;

                Ok(AutoMacros {
                    ident,
                    body: AutoMacrosType::EnumSinglePath { key, next_key },
                })
            } else {
                // Sth::Sth

                Ok(AutoMacros {
                    ident,
                    body: AutoMacrosType::EnumEmpty { key },
                })
            }
        } else {
            // Sth(...)

            let content;
            parenthesized!(content in input);

            let mut items = vec![];
            while !content.is_empty() {
                let value: Expr = content.parse()?;
                items.push(value);

                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }

            Ok(AutoMacros {
                ident,
                body: AutoMacrosType::Value { items },
            })
        }
    }
}
