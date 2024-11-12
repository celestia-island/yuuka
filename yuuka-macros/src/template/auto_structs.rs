use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::tools::StructsFlatten;

pub(crate) fn generate_structs_auto_macros(structs: StructsFlatten) -> Vec<TokenStream> {
    structs
        .iter()
        .map(|(ident, v, _extra_macros)| {
            let rules = v
                .iter()
                .map(|(name, ty, _, _)| {
                    quote! {
                        (#name { $($val: tt)+ }) => {
                            ::yuuka::auto!(#ty { $($val)+ })
                        };
                    }
                })
                .collect::<Vec<_>>();
            let rules = quote! {
                #(#rules)*
            };

            let ident = Ident::new(format!("__auto_{}", ident).as_str(), ident.span());
            quote! {
                #[doc(hidden)]
                macro_rules! #ident {
                    () => {};

                    #rules

                    ($name:ident $val:expr) => {
                        $val
                    };
                }
            }
        })
        .collect::<Vec<_>>()
}
