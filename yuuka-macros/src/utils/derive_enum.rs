use proc_macro2::TokenStream;
use std::{cell::RefCell, rc::Rc};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Expr, Ident, Token, TypePath,
};

use super::{DeriveEnumItems, DeriveVisibility, EnumMembers, ExtraMacros, StructName};

#[derive(Debug, Clone)]
pub struct DeriveEnum {
    pub visibility: DeriveVisibility,
    pub ident: StructName,
    pub items: EnumMembers,
    pub default_value: Option<Expr>,
    pub extra_macros: ExtraMacros,
}

impl DeriveEnum {
    pub fn pin_unique_id(&self, root_name: String, id: Rc<RefCell<usize>>) -> Self {
        let mut ret = self.clone();
        ret.ident = ret.ident.pin_unique_id(root_name, *id.borrow());
        *id.borrow_mut() += 1;
        ret
    }

    pub fn extend_attr_macros_before_derive(&self, extra_macros: Vec<TokenStream>) -> Self {
        let mut ret = self.clone();
        ret.extra_macros
            .extend_attr_macros_before_derive(extra_macros);
        ret
    }

    pub fn extend_derive_macros(&self, extra_macros: Vec<TypePath>) -> Self {
        let mut ret = self.clone();
        ret.extra_macros.extend_derive_macros(extra_macros);
        ret
    }

    pub fn extend_attr_macros_after_derive(&self, extra_macros: Vec<TokenStream>) -> Self {
        let mut ret = self.clone();
        ret.extra_macros
            .extend_attr_macros_after_derive(extra_macros);
        ret
    }

    pub fn extend_attr_macros_after_derive_recursive(
        &self,
        extra_macros: Vec<TokenStream>,
    ) -> Self {
        let mut ret = self.clone();
        ret.extra_macros
            .extend_attr_macros_after_derive_recursive(extra_macros);
        ret
    }
}

impl Parse for DeriveEnum {
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
                visibility,
                ident,
                items: content.items,
                default_value: Some(default_value),
                extra_macros,
            })
        } else {
            Ok(DeriveEnum {
                visibility,
                ident,
                items: content.items,
                default_value: None,
                extra_macros,
            })
        }
    }
}
