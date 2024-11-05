use proc_macro2::TokenStream;
use std::{cell::RefCell, rc::Rc};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, Token, TypePath,
};

use super::{DeriveStructItems, DeriveVisibility, ExtraMacros, StructMembers, StructName};

#[derive(Debug, Clone)]
pub struct DeriveStruct {
    pub visibility: DeriveVisibility,
    pub ident: StructName,
    pub items: StructMembers,
    pub extra_macros: ExtraMacros,
}

impl DeriveStruct {
    pub fn pin_unique_id(&self, root_name: String, id: Rc<RefCell<usize>>) -> Self {
        let mut ret = self.clone();
        ret.ident = ret.ident.pin_unique_id(root_name, *id.borrow());
        *id.borrow_mut() += 1;
        ret
    }

    pub fn extend_derive_macros(&self, extra_macros: Vec<TypePath>) -> Self {
        let mut ret = self.clone();
        ret.extra_macros.extend_derive_macros(extra_macros);
        ret
    }

    pub fn extend_attr_macros(&self, extra_macros: Vec<TokenStream>) -> Self {
        let mut ret = self.clone();
        ret.extra_macros.extend_attr_macros(extra_macros);
        ret
    }

    pub fn extend_attr_macros_recursive(&self, extra_macros: Vec<TokenStream>) -> Self {
        let mut ret = self.clone();
        ret.extra_macros.extend_attr_macros_recursive(extra_macros);
        ret
    }
}

impl Parse for DeriveStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let extra_macros = if input.peek(Token![#]) {
            input.parse::<ExtraMacros>()?
        } else {
            Default::default()
        };

        let visibility = if input.peek(Token![pub]) {
            input.parse::<Token![pub]>()?;
            DeriveVisibility::Public
        } else {
            DeriveVisibility::PublicOnCrate
        };

        let ident: StructName = if input.peek(Ident) {
            StructName::Named(input.parse()?)
        } else {
            StructName::Unnamed(None)
        };

        let content;
        braced!(content in input);
        let content: DeriveStructItems = content.parse()?;

        Ok(DeriveStruct {
            visibility,
            ident,
            items: content.items,
            extra_macros,
        })
    }
}
