use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    bracketed, parenthesized,
    parse::{Parse, ParseStream},
    token, Ident, Token, TypePath,
};

#[derive(Debug, Clone, Default)]
pub struct ExtraMacros {
    pub attr_macros_before_derive: Vec<TokenStream>,
    pub derive_macros: Vec<TypePath>,
    pub attr_macros_after_derive: Option<Vec<TokenStream>>,
    pub attr_macros_after_derive_recursive: Vec<TokenStream>,
}

impl ExtraMacros {
    pub fn extend_attr_macros_before_derive(&mut self, other: Vec<TokenStream>) {
        self.attr_macros_before_derive.extend(other);
    }

    pub fn extend_derive_macros(&mut self, other: Vec<TypePath>) {
        self.derive_macros.extend(other);
    }

    pub fn extend_attr_macros_after_derive(&mut self, other: Vec<TokenStream>) {
        if let Some(attr_macros_after_derive) = &mut self.attr_macros_after_derive {
            attr_macros_after_derive.extend(other);
        } else {
            self.attr_macros_after_derive = Some(other);
        }
    }

    pub fn extend_attr_macros_after_derive_recursive(&mut self, other: Vec<TokenStream>) {
        self.attr_macros_after_derive_recursive.extend(other);
    }
}

impl Parse for ExtraMacros {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attr_macros_before_derive = vec![];
        let mut derive_macros = vec![];
        let mut attr_macros_after_derive = vec![];
        let mut attr_macros_after_derive_recursive = vec![];

        let mut has_parsed_derive = false;

        while input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let bracked_content;
            bracketed!(bracked_content in input);

            let head_ident = bracked_content.parse::<Ident>()?;
            if head_ident == "derive" {
                if bracked_content.peek(token::Paren) {
                    let content;
                    parenthesized!(content in bracked_content);

                    while !content.is_empty() {
                        let item = content.parse::<TypePath>()?;
                        derive_macros.push(item);

                        if content.is_empty() {
                            break;
                        }
                        content.parse::<Token![,]>()?;
                    }
                }

                has_parsed_derive = true;
            } else if head_ident == "macros_recursive" && has_parsed_derive {
                let content;
                parenthesized!(content in bracked_content);

                let token_stream = content.parse::<TokenStream>()?;
                attr_macros_after_derive_recursive.push(token_stream);
            } else if !has_parsed_derive {
                let token_stream = bracked_content.parse::<TokenStream>()?;
                let token_stream = quote! {
                    #head_ident #token_stream
                };
                attr_macros_before_derive.push(token_stream);
            } else {
                let token_stream = bracked_content.parse::<TokenStream>()?;
                let token_stream = quote! {
                    #head_ident #token_stream
                };
                attr_macros_after_derive.push(token_stream);
            }
        }

        if !has_parsed_derive {
            Ok(Self {
                attr_macros_before_derive: attr_macros_after_derive,
                derive_macros: vec![],
                attr_macros_after_derive: None,
                attr_macros_after_derive_recursive,
            })
        } else {
            Ok(Self {
                attr_macros_before_derive,
                derive_macros,
                attr_macros_after_derive: Some(attr_macros_after_derive),
                attr_macros_after_derive_recursive,
            })
        }
    }
}
