use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    Ident,
};

#[derive(Debug, Clone)]
pub struct AutoMacros {
    pub ident: Ident,
    pub body: TokenStream,
}

impl Parse for AutoMacros {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            body: input.parse()?,
        })
    }
}
