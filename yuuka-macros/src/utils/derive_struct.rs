use std::{cell::RefCell, rc::Rc};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, Token,
};

use super::{DeriveStructItems, StructMembers, StructName};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeriveStructVisibility {
    Public,
    PublicOnCrate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeriveStruct {
    pub visibility: DeriveStructVisibility,
    pub ident: StructName,
    pub items: StructMembers,
}

impl DeriveStruct {
    pub fn pin_unique_id(&self, id: Rc<RefCell<usize>>) -> Self {
        let mut ret = self.clone();
        ret.ident = ret.ident.pin_unique_id(*id.borrow());
        *id.borrow_mut() += 1;
        ret
    }
}

impl Parse for DeriveStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let visibility = if input.peek(Token![pub]) {
            input.parse::<Token![pub]>()?;
            DeriveStructVisibility::Public
        } else {
            DeriveStructVisibility::PublicOnCrate
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
        })
    }
}
