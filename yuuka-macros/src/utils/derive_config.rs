use std::collections::HashMap;
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    token, Ident, Token, TypePath,
};

#[derive(Debug, Clone)]
pub struct DeriveConfig {
    pub ident: Ident,
    pub structs: Structs,
    pub enums: Enums,
}

#[derive(Debug, Clone)]
pub enum EnumValue {
    Empty,
    Tuple(Vec<TypePath>),
    Struct(StructMembers),
}

type Structs = HashMap<Ident, StructMembers>;
type StructMembers = HashMap<Ident, TypePath>;
type Enums = HashMap<Ident, EnumMembers>;
type EnumMembers = HashMap<Ident, EnumValue>;

fn merge_structs(source: &Structs, target: &mut Structs) {
    for (k, v) in source.iter() {
        if target.contains_key(&k) {
            panic!("duplicate key `{}`", k);
        }
        target.insert(k.clone(), v.clone());
    }
}

fn merge_enums(source: &Enums, target: &mut Enums) {
    for (k, v) in source.iter() {
        if target.contains_key(&k) {
            panic!("duplicate key `{}`", k);
        }
        target.insert(k.clone(), v.clone());
    }
}

impl Parse for DeriveConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut sub_structs: Structs = HashMap::new();
        let mut sub_enums: Enums = HashMap::new();

        if input.peek(Token![enum]) {
            // enum Ident { ... }
            input.parse::<Token![enum]>()?;
            let ident: Ident = input.parse()?;
            let content;
            braced!(content in input);

            let mut own_enum: EnumMembers = HashMap::new();

            while !content.is_empty() {
                let key = content.parse::<Ident>()?;

                let value = if content.peek(Token![,]) {
                    // Ident,
                    EnumValue::Empty
                } else if content.peek(token::Brace) {
                    // Ident { ... }
                    let sub_content;
                    braced!(sub_content in content);
                    let mut struct_members: StructMembers = HashMap::new();

                    while !sub_content.is_empty() {
                        let key = sub_content.parse::<Ident>()?;
                        sub_content.parse::<Token![:]>()?;

                        if sub_content.peek2(token::Brace) {
                            // ident: TypePath,
                            let ty = sub_content.parse::<TypePath>()?;
                            struct_members.insert(key, ty);
                        } else {
                            // ident: Ident { ... }
                            let item_content;
                            braced!(item_content in sub_content);

                            let content: DeriveConfig = item_content.parse()?;
                            struct_members.insert(
                                key,
                                syn::parse_str::<TypePath>(&content.ident.to_string())?,
                            );
                            merge_structs(&content.structs, &mut sub_structs);
                            merge_enums(&content.enums, &mut sub_enums);
                        }
                    }

                    EnumValue::Struct(struct_members)
                } else {
                    // Ident(...),
                    let sub_content;
                    parenthesized!(sub_content in content);
                    let mut tuple: Vec<TypePath> = Vec::new();

                    while !sub_content.is_empty() {
                        if content.peek2(Token![:]) {
                            // Ident: { ... }, ...
                            let item_content;
                            braced!(item_content in sub_content);

                            let content: DeriveConfig = item_content.parse()?;
                            merge_structs(&content.structs, &mut sub_structs);
                            merge_enums(&content.enums, &mut sub_enums);
                        } else {
                            // TypePath, ...
                            let ty = content.parse::<TypePath>()?;
                            tuple.push(ty);
                        }
                    }

                    EnumValue::Tuple(tuple)
                };

                own_enum.insert(key, value);

                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }

            sub_enums.insert(ident.clone(), own_enum);
            Ok(DeriveConfig {
                ident,
                structs: sub_structs,
                enums: sub_enums,
            })
        } else {
            let ident: Ident = input.parse()?;
            let content;
            braced!(content in input);

            let mut own_struct: StructMembers = HashMap::new();

            while !content.is_empty() {
                let key = content.parse::<Ident>()?;
                content.parse::<Token![:]>()?;

                if content.peek(token::Bracket) {
                    // sth: [...]
                    let bracket_level_content;
                    bracketed!(bracket_level_content in content);
                    let content: DeriveConfig = bracket_level_content.parse()?;
                    own_struct.insert(
                        key,
                        syn::parse_str::<TypePath>(&format!("Vec<{}>", content.ident))?,
                    );

                    merge_structs(&content.structs, &mut sub_structs);
                    merge_enums(&content.enums, &mut sub_enums);
                } else if content.peek(Token![enum]) {
                    // sth: enum { ... }
                    let content: DeriveConfig = content.parse()?;

                    own_struct.insert(key, syn::parse_str::<TypePath>(&content.ident.to_string())?);
                    merge_structs(&content.structs, &mut sub_structs);
                    merge_enums(&content.enums, &mut sub_enums);
                } else if !content.peek2(token::Brace) {
                    // sth: TypePath,
                    let ty = content.parse()?;
                    own_struct.insert(key, ty);
                } else {
                    // sth: Ident { ... }
                    let content: DeriveConfig = content.parse()?;
                    own_struct.insert(key, syn::parse_str::<TypePath>(&content.ident.to_string())?);
                    merge_structs(&content.structs, &mut sub_structs);
                    merge_enums(&content.enums, &mut sub_enums);
                }

                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }

            sub_structs.insert(ident.clone(), own_struct);
            Ok(DeriveConfig {
                ident,
                structs: sub_structs,
                enums: sub_enums,
            })
        }
    }
}
