#[cfg(test)]
mod test {
    use yuuka::{auto, derive_enum, derive_struct};

    #[test]
    fn basic_struct() {
        derive_struct!(Root {
            a: String,
            b: i32,
            c: f64,
            d: {
                e: String,
                f: i32,
            }
        });

        let obj = auto!(Root {
            a: "hello".to_string(),
            b: 42,
            c: 3.14,
            d: {
                e: "world".to_string(),
                f: 24,
            }
        });
        assert_eq!(obj.a, "hello");
        assert_eq!(obj.b, 42);
        assert_eq!(obj.c, 3.14);
        assert_eq!(obj.d.e, "world");
        assert_eq!(obj.d.f, 24);
    }

    #[test]
    fn basic_enum() {
        derive_enum!(enum Root {
            A,
            B(i32),
            C {
                a: String,
                b: i32,
            },
            D(enum {
                E,
                F(i32),
                G {
                    a: String,
                    b: i32,
                },
            })
        });

        assert_eq!(auto!(Root::A), Root::A);
        assert_eq!(auto!(Root::B(42)), Root::B(42));
        assert_eq!(
            auto!(Root::C {
                a: "hello".to_string(),
                b: 42
            }),
            Root::C {
                a: "hello".to_string(),
                b: 42
            }
        );
        assert_eq!(auto!(Root::D::E), Root::D(__Root::_Root_0_anonymous::E));
        assert_eq!(
            auto!(Root::D::F(42)),
            Root::D(__Root::_Root_0_anonymous::F(42))
        );
        assert_eq!(
            auto!(Root::D::G {
                a: "hello".to_string(),
                b: 42
            }),
            Root::D(__Root::_Root_0_anonymous::G {
                a: "hello".to_string(),
                b: 42
            })
        );
    }
}
