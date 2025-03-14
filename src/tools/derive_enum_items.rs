use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    token, Ident, Token, TypePath,
};

use crate::tools::ExtraTypeWrapper;

use super::{
    DeriveEnum, DeriveStruct, DeriveStructItems, EnumMembers, EnumValue, ExtraMacros, StructType,
};

#[derive(Debug, Clone)]
pub struct DeriveEnumItems {
    pub items: EnumMembers,
}

impl Parse for DeriveEnumItems {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut own_enum: EnumMembers = Vec::new();

        while !input.is_empty() {
            let extra_macros = if input.peek(Token![#]) {
                input.parse::<ExtraMacros>()?
            } else {
                Default::default()
            };

            let key = input.parse::<Ident>()?;

            let value = if input.peek(token::Brace) {
                // Ident { ... },
                let sub_content;
                braced!(sub_content in input);
                let content: DeriveStructItems = sub_content.parse()?;

                EnumValue::Struct(content.items)
            } else if input.peek(token::Paren) {
                // Ident(...),
                let sub_content;
                parenthesized!(sub_content in input);
                let mut tuple: Vec<(StructType, ExtraTypeWrapper)> = vec![];

                while !sub_content.is_empty() {
                    if sub_content.peek(token::Bracket) {
                        // Ident([...], ...),
                        let bracket_level_content;
                        bracketed!(bracket_level_content in sub_content);

                        if bracket_level_content.peek(Token![enum]) {
                            // Ident([enum Ident { ... }], ...),
                            // Ident([enum { ... }], ...),
                            let content: DeriveEnum = bracket_level_content.parse()?;
                            let content =
                                if let Some(derive_macros) = extra_macros.derive_macros.clone() {
                                    content
                                        .extend_derive_macros(derive_macros.derive_macros)
                                        .extend_attr_macros_recursive(
                                            derive_macros.attr_macros_recursive,
                                        )
                                } else {
                                    content
                                };

                            tuple.push((
                                StructType::InlineEnum(Box::new(content)),
                                ExtraTypeWrapper::Vec,
                            ));
                        } else {
                            // Ident([Ident { ... }], ...),
                            // Ident([{ ... }], ...),
                            let content: DeriveStruct = bracket_level_content.parse()?;
                            let content =
                                if let Some(derive_macros) = extra_macros.derive_macros.clone() {
                                    content
                                        .extend_derive_macros(derive_macros.derive_macros)
                                        .extend_attr_macros_recursive(
                                            derive_macros.attr_macros_recursive,
                                        )
                                } else {
                                    content
                                };

                            tuple.push((
                                StructType::InlineStruct(Box::new(content)),
                                ExtraTypeWrapper::Vec,
                            ));
                        }
                    } else if sub_content.peek(Token![enum]) {
                        // Ident(enum Ident { ... }, ...),
                        // Ident(enum { ... }, ...),
                        let content: DeriveEnum = sub_content.parse()?;
                        let content = if let Some(derive_macros) =
                            extra_macros.derive_macros.clone()
                        {
                            content
                                .extend_derive_macros(derive_macros.derive_macros)
                                .extend_attr_macros_recursive(derive_macros.attr_macros_recursive)
                        } else {
                            content
                        };

                        tuple.push((
                            StructType::InlineEnum(Box::new(content)),
                            ExtraTypeWrapper::Default,
                        ));
                    } else if sub_content.peek2(token::Brace) {
                        // Ident(Ident { ... }, ...),
                        // Ident({ ... }, ...),
                        let content: DeriveStruct = sub_content.parse()?;
                        let content = if let Some(derive_macros) =
                            extra_macros.derive_macros.clone()
                        {
                            content
                                .extend_derive_macros(derive_macros.derive_macros)
                                .extend_attr_macros_recursive(derive_macros.attr_macros_recursive)
                        } else {
                            content
                        };

                        tuple.push((
                            StructType::InlineStruct(Box::new(content)),
                            ExtraTypeWrapper::Default,
                        ));
                    } else {
                        // Ident (TypePath, ...),
                        let ty: TypePath = sub_content.parse()?;
                        tuple.push((StructType::Static(ty), ExtraTypeWrapper::Default));
                    }

                    if sub_content.peek(Token![,]) {
                        sub_content.parse::<Token![,]>()?;
                    }
                }

                EnumValue::Tuple(tuple)
            } else {
                // Ident,
                EnumValue::Empty
            };

            own_enum.push((key, value, extra_macros));

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(DeriveEnumItems { items: own_enum })
    }
}
