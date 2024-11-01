use proc_macro::TokenStream;
use quote::quote;
use std::{cell::RefCell, rc::Rc};
use syn::parse_macro_input;

mod utils;
use utils::{
    flatten, DefaultValue, DeriveEnum, DeriveStruct, DeriveVisibility, EnumValueFlatten, StructName,
};

#[proc_macro]
pub fn derive_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveStruct);

    let is_public = input.visibility == DeriveVisibility::Public;
    let root_ident = match input.ident.clone() {
        StructName::Named(v) => v,
        StructName::Unnamed(_) => {
            panic!("Unnamed root struct is not supported");
        }
    };
    let mod_ident = syn::Ident::new(&format!("__{}", root_ident), root_ident.span());
    let (structs, enums) = flatten(
        root_ident.to_string(),
        Rc::new(RefCell::new(0)),
        utils::DeriveBox::Struct(input.clone()),
        vec![],
    )
    .expect("Failed to flatten");

    let structs = structs
        .iter()
        .map(|(ident, v, extra_macros)| {
            let keys = v
                .iter()
                .map(|(key, ty, _default_value, extra_macros)| {
                    let extra_macros = extra_macros
                        .iter()
                        .map(|content| {
                            quote! {
                                #[#content]
                            }
                        })
                        .collect::<Vec<_>>();

                    quote! {
                        #(#extra_macros)*
                        pub #key: #ty,
                    }
                })
                .collect::<Vec<_>>();
            let extra_macros = extra_macros
                .iter()
                .map(|content| {
                    quote! {
                        #[#content]
                    }
                })
                .collect::<Vec<_>>();

            if v.iter()
                .all(|(_, _, default_value, _)| default_value == &DefaultValue::None)
            {
                quote! {
                    #[derive(Debug, Clone, Default)]
                    #(#extra_macros)*
                    pub struct #ident {
                        #( #keys )*
                    }
                }
            } else {
                let default_values = v
                    .iter()
                    .map(|(key, _, default_value, _)| match default_value {
                        DefaultValue::None => quote! {
                            #key: Default::default(),
                        },
                        DefaultValue::Single(v) => quote! {
                            #key: #v,
                        },
                        DefaultValue::Array(v) => quote! {
                            #key: vec![#(#v),*],
                        },
                    })
                    .collect::<Vec<_>>();

                quote! {
                    #[derive(Debug, Clone)]
                    #(#extra_macros)*
                    pub struct #ident {
                        #( #keys )*
                    }

                    impl Default for #ident {
                        fn default() -> Self {
                            Self {
                                #( #default_values )*
                            }
                        }
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let enums = enums
        .iter()
        .map(|(k, v, default_value, extra_macros)| {
            let keys = v
                .iter()
                .map(|(key, ty, extra_macros)| {
                    let extra_macros = extra_macros
                        .iter()
                        .map(|content| {
                            quote! {
                                #[#content]
                            }
                        })
                        .collect::<Vec<_>>();

                    match ty {
                        EnumValueFlatten::Empty => quote! {
                            #(#extra_macros)*
                            #key,
                        },
                        EnumValueFlatten::Tuple(v) => {
                            quote! {
                            #(#extra_macros)*
                                #key(#(#v),*),
                            }
                        }
                        EnumValueFlatten::Struct(v) => {
                            let keys = v
                                .iter()
                                .map(|(key, ty, _default_value, extra_macros)| {
                                    let extra_macros = extra_macros
                                        .iter()
                                        .map(|content| {
                                            quote! {
                                                #[#content]
                                            }
                                        })
                                        .collect::<Vec<_>>();

                                    quote! {
                                        #(#extra_macros)*
                                        #key: #ty,
                                    }
                                })
                                .collect::<Vec<_>>();

                            quote! {
                                #(#extra_macros)*
                                #key {
                                    #( #keys )*
                                },
                            }
                        }
                    }
                })
                .collect::<Vec<_>>();
            let default_value = if let DefaultValue::Single(default_value) = default_value {
                quote! {
                    impl Default for #k {
                        fn default() -> Self {
                            #default_value
                        }
                    }
                }
            } else {
                quote! {
                    impl Default for #k {
                        fn default() -> Self {
                            unimplemented!("Default value for enum is not implemented");
                        }
                    }
                }
            };
            let extra_macros = extra_macros
                .iter()
                .map(|content| {
                    quote! {
                        #[#content]
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                #[derive(Debug, Clone)]
                #(#extra_macros)*
                pub enum #k {
                    #( #keys )*
                }

                #default_value
            }
        })
        .collect::<Vec<_>>();

    let ret = if is_public {
        quote! {
            #[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
            pub mod #mod_ident {
                use super::*;

                #( #structs )*
                #( #enums )*
            }

            pub use #mod_ident::*;
        }
    } else {
        quote! {
            #[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
            pub(crate) mod #mod_ident {
                use super::*;

                #( #structs )*
                #( #enums )*
            }

            pub(crate) use #mod_ident::*;
        }
    };

    ret.into()
}

#[proc_macro]
pub fn derive_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveEnum);

    let is_public = input.visibility == DeriveVisibility::Public;
    let root_ident = match input.ident.clone() {
        StructName::Named(v) => v,
        StructName::Unnamed(_) => {
            panic!("Unnamed root struct is not supported");
        }
    };
    let mod_ident = syn::Ident::new(&format!("__{}", root_ident), root_ident.span());
    let (structs, enums) = flatten(
        root_ident.to_string(),
        Rc::new(RefCell::new(0)),
        utils::DeriveBox::Enum(input.clone()),
        vec![],
    )
    .expect("Failed to flatten");

    let structs = structs
        .iter()
        .map(|(ident, v, extra_macros)| {
            let keys = v
                .iter()
                .map(|(key, ty, _default_value, extra_macros)| {
                    let extra_macros = extra_macros
                        .iter()
                        .map(|content| {
                            quote! {
                                #[#content]
                            }
                        })
                        .collect::<Vec<_>>();

                    quote! {
                        #(#extra_macros)*
                        pub #key: #ty,
                    }
                })
                .collect::<Vec<_>>();
            let extra_macros = extra_macros
                .iter()
                .map(|content| {
                    quote! {
                        #[#content]
                    }
                })
                .collect::<Vec<_>>();

            if v.iter()
                .all(|(_, _, default_value, _)| default_value == &DefaultValue::None)
            {
                quote! {
                    #[derive(Debug, Clone, Default)]
                    #(#extra_macros)*
                    pub struct #ident {
                        #( #keys )*
                    }
                }
            } else {
                let default_values = v
                    .iter()
                    .map(|(key, _, default_value, _)| match default_value {
                        DefaultValue::None => quote! {
                            #key: Default::default(),
                        },
                        DefaultValue::Single(v) => quote! {
                            #key: #v,
                        },
                        DefaultValue::Array(v) => quote! {
                            #key: vec![#(#v),*],
                        },
                    })
                    .collect::<Vec<_>>();

                quote! {
                    #[derive(Debug, Clone)]
                    #(#extra_macros)*
                    pub struct #ident {
                        #( #keys )*
                    }

                    impl Default for #ident {
                        fn default() -> Self {
                            Self {
                                #( #default_values )*
                            }
                        }
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let enums = enums
        .iter()
        .map(|(k, v, default_value, extra_macros)| {
            let keys = v
                .iter()
                .map(|(key, ty, extra_macros)| {
                    let extra_macros = extra_macros
                        .iter()
                        .map(|content| {
                            quote! {
                                #[#content]
                            }
                        })
                        .collect::<Vec<_>>();

                    match ty {
                        EnumValueFlatten::Empty => quote! {
                            #(#extra_macros)*
                            #key,
                        },
                        EnumValueFlatten::Tuple(v) => {
                            quote! {
                            #(#extra_macros)*
                                #key(#(#v),*),
                            }
                        }
                        EnumValueFlatten::Struct(v) => {
                            let keys = v
                                .iter()
                                .map(|(key, ty, _default_value, extra_macros)| {
                                    let extra_macros = extra_macros
                                        .iter()
                                        .map(|content| {
                                            quote! {
                                                #[#content]
                                            }
                                        })
                                        .collect::<Vec<_>>();

                                    quote! {
                                        #(#extra_macros)*
                                        #key: #ty,
                                    }
                                })
                                .collect::<Vec<_>>();

                            quote! {
                                #(#extra_macros)*
                                #key {
                                    #( #keys )*
                                },
                            }
                        }
                    }
                })
                .collect::<Vec<_>>();
            let default_value = if let DefaultValue::Single(default_value) = default_value {
                quote! {
                    impl Default for #k {
                        fn default() -> Self {
                            #default_value
                        }
                    }
                }
            } else {
                quote! {
                    impl Default for #k {
                        fn default() -> Self {
                            unimplemented!("Default value for enum is not implemented");
                        }
                    }
                }
            };
            let extra_macros = extra_macros
                .iter()
                .map(|content| {
                    quote! {
                        #[#content]
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                #[derive(Debug, Clone)]
                #(#extra_macros)*
                pub enum #k {
                    #( #keys )*
                }

                #default_value
            }
        })
        .collect::<Vec<_>>();

    let ret = if is_public {
        quote! {
            #[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
            pub mod #mod_ident {
                use super::*;

                #( #structs )*
                #( #enums )*
            }

            pub use #mod_ident::*;
        }
    } else {
        quote! {
            #[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
            pub(crate) mod #mod_ident {
                use super::*;

                #( #structs )*
                #( #enums )*
            }

            pub(crate) use #mod_ident::*;
        }
    };

    ret.into()
}
