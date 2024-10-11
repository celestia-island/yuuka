use std::{cell::RefCell, rc::Rc};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    Ident, Token,
};

use super::{DeriveStructItems, ExtraMacros, StructMembers, StructName};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeriveStructVisibility {
    Public,
    PublicOnCrate,
}

#[derive(Debug, Clone)]
pub struct DeriveStruct {
    pub visibility: DeriveStructVisibility,
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

    pub fn extend_extra_macros(&self, extra_macros: ExtraMacros) -> Self {
        let mut ret = self.clone();
        ret.extra_macros.extend(extra_macros);
        ret
    }
}

impl Parse for DeriveStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut extra_macros = vec![];
        while input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let content;
            bracketed!(content in input);

            extra_macros.push(content.parse()?);
        }

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
            extra_macros,
        })
    }
}
