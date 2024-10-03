use syn::{
    parse::{Parse, ParseStream},
    Ident,
};

#[derive(Debug, Clone)]
pub struct Auto {
    pub ident: Ident,
}

impl Parse for Auto {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Auto { ident: todo!() })
    }
}
