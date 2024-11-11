use proc_macro2::TokenStream;
use syn::{
    braced,
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
        let ident = input.parse()?;
        let body;
        braced!(body in input);

        Ok(Self {
            ident,
            body: body.parse()?,
        })
    }
}
