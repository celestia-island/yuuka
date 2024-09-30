use proc_macro2::Span;
use std::collections::HashMap;
use syn::{
    parse::{Parse, ParseStream},
    Ident, Type,
};

pub struct DeriveConfig {
    pub ident: Ident,
    pub members: Components,
}

type Components = HashMap<Ident, Type>;

impl Parse for DeriveConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: Ident::new("Config", Span::call_site()),
            members: HashMap::new(),
        })
    }
}
