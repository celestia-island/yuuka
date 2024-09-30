use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod utils;

#[proc_macro]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as utils::derive_config::DeriveConfig);
    dbg!(input);

    let ret = quote! {};
    ret.into()
}
