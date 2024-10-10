use std::{cell::RefCell, rc::Rc};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    Expr, Ident, Token,
};

use super::{DeriveEnumItems, EnumMembers, ExtraMacros, StructName};

#[derive(Debug, Clone)]
pub struct DeriveEnum {
    pub ident: StructName,
    pub items: EnumMembers,
    pub default_value: Option<Expr>,
    pub extra_macros: ExtraMacros,
}

impl DeriveEnum {
    pub fn pin_unique_id(&self, id: Rc<RefCell<usize>>) -> Self {
        let mut ret = self.clone();
        ret.ident = ret.ident.pin_unique_id(*id.borrow());
        *id.borrow_mut() += 1;
        ret
    }
}

impl Parse for DeriveEnum {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut extra_macros = vec![];
        while input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let content;
            bracketed!(content in input);

            extra_macros.push(content.parse()?);
        }

        input.parse::<Token![enum]>()?;
        let ident: StructName = if input.peek(Ident) {
            StructName::Named(input.parse()?)
        } else {
            StructName::Unnamed(None)
        };
        let content;
        braced!(content in input);
        let content: DeriveEnumItems = content.parse()?;

        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let default_value = input.parse::<Expr>()?;

            Ok(DeriveEnum {
                ident,
                items: content.items,
                default_value: Some(default_value),
                extra_macros,
            })
        } else {
            Ok(DeriveEnum {
                ident,
                items: content.items,
                default_value: None,
                extra_macros,
            })
        }
    }
}
