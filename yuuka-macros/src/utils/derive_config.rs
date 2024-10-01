use std::collections::HashMap;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    token, Ident, Token, TypePath,
};

#[derive(Debug, Clone)]
pub struct DeriveConfig {
    pub ident: Ident,
    pub structs: Structs,
}

type Structs = HashMap<Ident, StructMembers>;
type StructMembers = HashMap<Ident, TypePath>;

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

            if content.peek(token::Bracket) {
                dbg!("bracketed");
                let bracket_level_content;
                let _ = bracketed!(bracket_level_content in content);
                let content: DeriveConfig = bracket_level_content.parse()?;
                dbg!(content.ident.clone());
                own_struct.insert(
                    key,
                    syn::parse_str::<TypePath>(&format!("Vec<{}>", content.ident))?,
                );

                // Merge the sub-structs into the current struct
                for (k, v) in content.structs {
                    // If there is a duplicate key, an error is reported
                    if own_struct.contains_key(&k) {
                        return Err(syn::Error::new(k.span(), format!("duplicate key `{}`", k)));
                    }
                    sub_structs.insert(k, v);
                }
            } else if !content.peek2(token::Brace) {
                let ty = content.parse()?;
                own_struct.insert(key, ty);
            } else {
                let content: DeriveConfig = content.parse()?;
                own_struct.insert(key, syn::parse_str::<TypePath>(&content.ident.to_string())?);

                // Merge the sub-structs into the current struct
                for (k, v) in content.structs {
                    // If there is a duplicate key, an error is reported
                    if own_struct.contains_key(&k) {
                        return Err(syn::Error::new(k.span(), format!("duplicate key `{}`", k)));
                    }
                    sub_structs.insert(k, v);
                }
            }

            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        sub_structs.insert(ident.clone(), own_struct);
        Ok(DeriveConfig {
            ident,
            structs: sub_structs,
        })
    }
}
