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
}
