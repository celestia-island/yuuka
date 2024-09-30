use std::collections::HashMap;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, Token,
};

#[derive(Debug)]
pub struct DeriveConfig {
    pub ident: Ident,
    pub structs: Structs,
}

type Structs = HashMap<Ident, StructMembers>;
type StructMembers = HashMap<Ident, Ident>;

impl Parse for DeriveConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let content;
        let _ = braced!(content in input);

        let mut sub_structs: Structs = HashMap::new();
        let mut own_struct: StructMembers = HashMap::new();
        while !content.is_empty() {
            let key = content.parse::<Ident>()?;
            content.parse::<Token![:]>()?;

            if content.peek(Ident) {
                let ty = content.parse()?;
                own_struct.insert(key, ty);
            } else {
                let content: DeriveConfig = content.parse()?;
                own_struct.insert(key, content.ident.clone());

                // Merge the sub-structs into the current struct
                for (k, v) in content.structs {
                    // If there is a duplicate key, an error is reported
                    if own_struct.contains_key(&k) {
                        return Err(syn::Error::new(k.span(), format!("duplicate key `{}`", k)));
                    }
                    sub_structs.insert(k, v);
                }
            }

            if content.parse::<Token![,]>().is_err() {
                break;
            }
        }

        sub_structs.insert(ident.clone(), own_struct);
        Ok(DeriveConfig {
            ident,
            structs: sub_structs,
        })
    }
}
