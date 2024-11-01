#[cfg(test)]
mod test {
    use yuuka::derive_struct;

    #[test]
    fn default_struct() {
        derive_struct!(Root {
            a: String = "hello".to_string(),
            b: i32 = 42,
            c: String,
        });

        let val = Root::default();
        assert_eq!(val.a, "hello".to_string());
        assert_eq!(val.b, 42);
        assert_eq!(val.c, String::default());
    }

    #[test]
    fn empty_default_array() {
        derive_struct!(Root {
            a: [Item {
                b: String = "hello".to_string()
            }]
        });

        let val = Root::default();
        assert_eq!(val.a.len(), 0);
    }

    #[test]
    fn default_array() {
        derive_struct!(Root {
            a: [Item {
                b: String = "hello".to_string()
            }] = vec![Item {
                b: "world".to_string()
            }]
        });

        let mut val = Root::default();
        assert_eq!(val.a.len(), 1);
        assert_eq!(val.a[0].b, "world");
        val.a.push(Default::default());
        assert_eq!(val.a.len(), 2);
        assert_eq!(val.a[1].b, "hello");
    }

    #[test]
    fn default_enum() {
        derive_struct!(
            #[derive(PartialEq)]
            Root {
                a: enum Member {
                    Momoi,
                    Midori,
                    Yuzu,
                    Arisu,
                } = Midori
            }
        );

        let val = Root::default();
        assert_eq!(val.a, Member::Midori);
    }

    #[test]
    fn default_enum_array() {
        derive_struct!(
            #[derive(PartialEq)]
            Root {
                a: [enum Member {
                    Momoi,
                    Midori,
                    Yuzu,
                    Arisu,
                } = Midori] = vec![Member::Arisu]
            }
        );

        let mut val = Root::default();
        assert_eq!(val.a.len(), 1);
        val.a.push(Default::default());
        assert_eq!(val.a.len(), 2);
        assert_eq!(val.a[0], Member::Arisu);
        assert_eq!(val.a[1], Member::Midori);
    }
}
