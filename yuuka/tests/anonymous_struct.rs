#[cfg(test)]
mod test {
    use yuuka::{auto, derive_struct};

    #[test]
    fn derive_struct_anonymously() {
        derive_struct!(Root {
            a: [{ b: String }]
        });

        let _ = auto! {
            Root {
                a: vec![{
                    b: "hello".to_string(),
                }],
            }
        };
    }

    #[test]
    fn derive_enum_anonymously() {
        derive_struct!(Root {
            a: enum {
                Momoi,
                Midori,
                Yuzu,
                Arisu [{
                    r#type: enum {
                        AL1S,
                        Key,
                    }
                }],
            }
        });

        let _ = auto! {
            Root {
                a: Arisu {
                    r#type: vec![Key, AL1S],
                },
            }
        };
    }

    #[test]
    fn create_default_struct() {
        derive_struct!(Profile {
            id: uuid::Uuid,
            name: String,
            email: String,
            extra_profile: {
                age: Option<usize>,
                sex: enum {
                    Male, Female, Other(String)
                },
                points: usize
            }
        });

        impl Default for Profile {
            fn default() -> {
                auto! {
                    Profile {
                        id: uuid::Uuid::new_v4(),
                        name: "HOMO",
                        email: "114514@homo.io",
                        extra_profile: {
                            age: Some(24),
                            sex: Male,
                            points: 1919810
                        }
                    }
                }
            }
        }
    }
}
