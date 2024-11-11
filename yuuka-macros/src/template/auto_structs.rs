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
                        (#name: $val:block, $($next:tt)*) => {
                            #name: $crate::auto!(#ty { $val }),
                            #ident!($($next)*)
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

                    ($($name:ident: $val:expr,)+ $($next:tt)*) => {
                        $($name: $val.into(),)+
                        #ident!($($next)*)
                    };
                    (..$expr:expr, $($next:tt)*) => {
                        ..$expr,
                        #ident!($($next)*)
                    };

                    #rules
                }
            }
        })
        .collect::<Vec<_>>()
}
