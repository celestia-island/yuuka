use proc_macro2::TokenStream;
use quote::quote;

use crate::tools::StructsFlatten;

pub(crate) fn generate_structs_auto_macros(structs: StructsFlatten) -> Vec<TokenStream> {
    structs
        .iter()
        .map(|(ident, v, _extra_macros)| {
            let rules = v
                .iter()
                .map(|(name, ty, _, _)| {
                    quote! {
                        (#name: $val:block,) => {
                            #name: #ty { $val },
                        };
                    }
                })
                .collect::<Vec<_>>();
            let rules = quote! {
                #(#rules)*
            };

            quote! {
                macro_rules! #ident {
                    #rules

                    ($name:ident: $val:expr,) => {
                        $name: $val,
                    };
                    (..$expr:expr,) => {
                        ..$expr,
                    };
                }
            }
        })
        .collect::<Vec<_>>()
}
