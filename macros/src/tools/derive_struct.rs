use proc_macro2::TokenStream;
use std::{cell::RefCell, rc::Rc};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, Token, TypePath,
};

use super::{
    reject_attributes_before_derive, DeriveStructItems, DeriveVisibility, ExtraMacros,
    StructMembers, StructName,
};

#[derive(Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn type_attribute_before_derive_is_rejected_not_dropped() {
        // A type-level attribute placed before `#[derive(..)]` used to be
        // parsed and then silently dropped by flatten (finding Y1); it must
        // now be a loud error pointing at the attribute.
        let err = match syn::parse2::<DeriveStruct>(quote! {
            #[serde(rename_all = "camelCase")]
            #[derive(Debug, Clone)]
            pub Root {
                nick_name: String,
            }
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
    fn type_attribute_without_any_derive_is_rejected_too() {
        // Even with no `#[derive(..)]` at all, a type-level attribute never
        // reaches the generated type — reject it instead of dropping it.
        assert!(syn::parse2::<DeriveStruct>(quote! {
            #[serde(rename_all = "camelCase")]
            pub Root {
                nick_name: String,
            }
        })
        .is_err());
    }

    #[test]
    fn type_attribute_after_derive_still_parses() {
        syn::parse2::<DeriveStruct>(quote! {
            #[derive(Debug, Clone)]
            #[serde(rename_all = "camelCase")]
            pub Root {
                nick_name: String,
            }
        })
        .unwrap();
    }

    #[test]
    fn field_attribute_before_field_derive_is_still_accepted() {
        // Field-level attributes keep their documented placement: they apply
        // to the generated field and must keep parsing without a derive.
        syn::parse2::<DeriveStruct>(quote! {
            #[derive(Debug, Clone)]
            pub Root {
                #[serde(rename = "location")]
                live_in: String,
            }
        })
        .unwrap();
    }

    #[test]
    fn inline_struct_attribute_before_derive_is_rejected_too() {
        // Bracketed inline types go through the same `DeriveStruct` parser:
        // their pre-derive attributes would be dropped by flatten all the
        // same, so they must be rejected here too.
        assert!(syn::parse2::<DeriveStruct>(quote! {
            #[derive(Debug, Clone)]
            pub Root {
                location: [
                    #[serde(rename_all = "UPPERCASE")]
                    #[derive(Clone)]
                    Inner {
                        city: String,
                    }
                ],
            }
        })
        .is_err());
    }

    #[test]
    fn inline_struct_compliant_placement_still_parses() {
        // The compliant inline-type placement (attributes after the inline
        // `#[derive(..)]`) keeps parsing.
        syn::parse2::<DeriveStruct>(quote! {
            #[derive(Debug, Clone)]
            pub Root {
                location: [
                    #[derive(Clone)]
                    #[serde(rename_all = "UPPERCASE")]
                    Inner {
                        city: String,
                    }
                ],
            }
        })
        .unwrap();
    }
}
