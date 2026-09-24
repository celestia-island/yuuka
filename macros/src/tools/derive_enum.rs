use proc_macro2::TokenStream;
use std::{cell::RefCell, rc::Rc};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Expr, Ident, Token, TypePath,
};

use super::{
    reject_attributes_before_derive, DeriveEnumItems, DeriveVisibility, EnumMembers, ExtraMacros,
    StructName,
};

#[derive(Clone)]
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

impl Parse for DeriveEnum {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let extra_macros = if input.peek(Token![#]) {
            let extra_macros: ExtraMacros = input.parse()?;
            reject_attributes_before_derive(&extra_macros)?;
            extra_macros
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

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn enum_type_attribute_before_derive_is_rejected_not_dropped() {
        let err = match syn::parse2::<DeriveEnum>(quote! {
            #[serde(rename_all = "snake_case")]
            #[derive(Debug, Clone)]
            pub enum Root {
                SaibaMomoi,
                SaibaMidori,
            } = SaibaMomoi
        }) {
            Ok(_) => panic!("pre-derive type attribute must not parse"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("must be placed after `#[derive(..)]`"),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn enum_type_attribute_after_derive_still_parses() {
        syn::parse2::<DeriveEnum>(quote! {
            #[derive(Debug, Clone)]
            #[serde(rename_all = "snake_case")]
            pub enum Root {
                SaibaMomoi,
                SaibaMidori,
            } = SaibaMomoi
        })
        .unwrap();
    }
}
