use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    token, Expr, Ident, Token,
};

#[derive(Debug, Clone)]
pub enum AutoMacrosType {
    Struct(Vec<(Ident, TokenStream)>),
    Enum,
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
            let content;
            braced!(content in input);

            let mut tokens = vec![];
            while !content.is_empty() {
                let key: Ident = content.parse()?;
                content.parse::<Token![:]>()?;

                if content.peek(token::Brace) {
                    let inner_content;
                    braced!(inner_content in content);
                    let inner_content: TokenStream = inner_content.parse()?;
                    tokens.push((
                        key,
                        quote! {
                            { #inner_content }
                        },
                    ));
                } else if content.peek(token::Bracket) {
                    let inner_content;
                    bracketed!(inner_content in content);
                    let inner_content: TokenStream = inner_content.parse()?;
                    tokens.push((
                        key,
                        quote! {
                             [ #inner_content ]
                        },
                    ));
                } else if content.peek(token::Paren) {
                    let inner_content;
                    parenthesized!(inner_content in content);
                    let inner_content: TokenStream = inner_content.parse()?;
                    tokens.push((
                        key,
                        quote! {
                            ( #inner_content )
                        },
                    ));
                } else {
                    let value: Expr = content.parse()?;
                    tokens.push((
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
                body: AutoMacrosType::Struct(tokens),
            })
        } else {
            Ok(AutoMacros {
                ident,
                body: AutoMacrosType::Enum,
            })
        }
    }
}
