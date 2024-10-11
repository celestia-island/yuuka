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
    fn derive_struct_anonymously_multiple() {
        derive_struct!(Root {
            a: {
                b: String
            },
            c: {
                d: f64
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
    fn derive_enum_anonymously_multiple() {
        derive_struct!(Root {
            a: enum {
                Momoi,
                Midori,
                Yuzu,
                Arisu,
            },
            b: enum {
                Apple,
                Pen,
                Pineapple,
                ApplePen,
            }
        });
    }

    #[test]
    fn derive_struct_anonymously_with_braces() {
        derive_struct!(Root {
            a: {
                b: String,
                c: {
                    d: f64 = std::f64::consts::PI,
                    e: {
                        f: bool = false,
                    },
                },
                g: {
                    h: i32 = -114514,
                }
            },
            i: {
                j: String = "いいよ，こいよ".to_string(),
            }
        });

        let root = Root::default();
        assert_eq!(root.a.b, String::default());
        assert_eq!(root.a.c.d, std::f64::consts::PI);
        assert!(!root.a.c.e.f);
        assert_eq!(root.a.g.h, -114514);
        assert_eq!(root.i.j, "いいよ，こいよ".to_string());
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
            a: Vec<String> = vec!["Hello".to_string()],
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
        derive_struct!(Root2 {
            a: [enum {
                Momoi,
                Midori,
                Yuzu,
                Arisu,
            } = Midori],
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
            } = Midori],
        });
        derive_struct!(Root2 {
            a: [enum {
                Momoi,
                Midori,
                Yuzu,
                Arisu(usize),
            } = Arisu(233)],
        });

        let mut root = Root::default();
        root.a.push(Default::default());
        assert_eq!(root.a, vec![__Root::_Root_0_anonymous::Midori]);
        let mut root2 = Root2::default();
        root2.a.push(Default::default());
        assert_eq!(root2.a, vec![__Root2::_Root2_0_anonymous::Arisu(233)]);
    }
}
