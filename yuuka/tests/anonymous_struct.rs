#[cfg(test)]
mod test {
    use yuuka::derive_struct;

    #[test]
    fn derive_struct_anonymously() {
        derive_struct!(Root {
            a: {
                b: String
            }
        });
    }

    #[test]
    fn derive_enum_anonymously() {
        derive_struct!(Root {
            a: enum {
                Momoi,
                Midori,
                Yuzu,
                Arisu,
            }
        });
    }

    #[test]
    fn derive_struct_anonymously_with_braces() {
        derive_struct!(Root {
            a: {
                b: String,
                c: {
                    d: f64,
                    e: {
                        f: bool,
                    },
                },
            }
        });
    }

    #[test]
    fn derive_struct_anonymously_with_array() {
        derive_struct!(Root {
            a: [{
                b: String,
            }]
        });
    }

    #[test]
    fn derive_struct_anonymously_with_enum_array() {
        derive_struct!(Root {
            a: [enum {
                Momoi,
                Midori,
                Yuzu,
                Arisu,
            }],
        });
    }

    #[test]
    fn derive_struct_anonymously_with_default_value() {
        derive_struct!(Root {
            a: {
                b: String = "Hello".to_string()
            },
        });
    }

    #[test]
    fn derive_struct_anonymously_with_default_array() {
        derive_struct!(Root {
            a: [String] = ["Hello".to_string()],
        });
    }

    #[test]
    fn derive_struct_anonymously_with_default_enum() {
        derive_struct!(Root {
            a: enum {
                Momoi,
                Midori,
                Yuzu,
                Arisu,
            } = Midori,
        });
    }

    #[test]
    fn derive_struct_anonymously_with_default_enum_array() {
        derive_struct!(Root {
            a: [enum {
                Momoi,
                Midori,
                Yuzu,
                Arisu,
            } = Midori] = [Arisu],
        });
    }
}
