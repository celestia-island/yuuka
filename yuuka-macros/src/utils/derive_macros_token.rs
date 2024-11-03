use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    bracketed, parenthesized,
    parse::{Parse, ParseStream},
    Ident, Token,
};

#[derive(Debug, Clone, Default)]
pub struct ExtraMacros {
    pub derive_macros: Vec<Ident>,
    pub attr_macros: Vec<TokenStream>,
}

impl ExtraMacros {
    pub fn extend_derive_macros(&mut self, other: Vec<Ident>) {
        self.derive_macros.extend(other);
    }
}

impl Parse for ExtraMacros {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut derive_macros = vec![];
        let mut attr_macros = vec![];

        while input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let bracked_content;
            bracketed!(bracked_content in input);

            let head_ident = bracked_content.parse::<Ident>()?;
            if head_ident == "derive" {
                let content;
                parenthesized!(content in bracked_content);

                while !content.is_empty() {
                    let item = content.parse::<Ident>()?;
                    derive_macros.push(item);

                    if content.is_empty() {
                        break;
                    }
                    content.parse::<Token![,]>()?;
                }
            } else {
                let token_stream = bracked_content.parse::<TokenStream>()?;
                let token_stream = quote! {
                    #head_ident #token_stream
                };
                attr_macros.push(token_stream);
            }
        }

        Ok(Self {
            derive_macros,
            attr_macros,
        })
    }
}
