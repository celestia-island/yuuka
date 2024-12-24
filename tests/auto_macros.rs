use yuuka::{auto, derive_enum, derive_struct};

#[test]
fn basic_struct() {
    derive_struct!(Root {
        a: String,
        b: i32,
        c: f64,
        d: {
            e: String = "world".to_string(),
            f: i32,
        }
    });

    let obj = auto!(Root {
        a: "hello".to_string(),
        b: 42,
        c: std::f64::consts::PI,
        d: {
            f: 24,
            ..Default::default(),
        }
    });
    assert_eq!(obj.a, "hello");
    assert_eq!(obj.b, 42);
    assert_eq!(obj.c, std::f64::consts::PI);
    assert_eq!(obj.d.e, "world");
    assert_eq!(obj.d.f, 24);
}

#[test]
fn basic_enum() {
    derive_enum!(
        #[derive(PartialEq)]
        enum Root {
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
        }
    );

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

#[test]
fn multi_level_enum() {
    derive_enum!(
        #[derive(PartialEq)]
        enum A {
            B(enum {
                C(enum {
                    D(enum {
                        E(enum {
                            F,
                            G(String),
                        })
                    })
                })
            })
        }
    );

    assert_eq!(
        auto!(A::B::C::D::E::F),
        A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(
            _A_3_anonymous::F
        ))))
    );
    assert_eq!(
        auto!(A::B::C::D::E::G("いいよ！こいよ！".to_string())),
        A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(
            _A_3_anonymous::G("いいよ！こいよ！".to_string())
        ))))
    );
}

#[test]
fn mixed_auto() {
    derive_struct!(
        #[derive(PartialEq)]
        Root {
            outer: {
                a: enum B {
                    C {
                        c: i32,
                        d: f64,
                    }
                }
            }
        }
    );

    assert_eq!(
        auto!(Root {
            outer: {
                a: auto!(B::C { c: 42, d: std::f64::consts::PI })
            }
        }),
        Root {
            outer: _Root_0_anonymous {
                a: B::C {
                    c: 42,
                    d: std::f64::consts::PI
                }
            }
        }
    );
}

#[test]
fn default_struct_auto() {
    derive_struct!(Root {
        a: String,
        b: i32,
        c: f64,
        d: {
            e: String = "world".to_string(),
            f: i32,
        }
    });

    let obj = auto!(Root {
        a: "hello".to_string(),
        b: 42,
        c: std::f64::consts::PI,
        d: {
            f: 24,
            ..Default::default(),
        }
    });
    assert_eq!(obj.a, "hello");
    assert_eq!(obj.b, 42);
    assert_eq!(obj.c, std::f64::consts::PI);
    assert_eq!(obj.d.e, "world");
    assert_eq!(obj.d.f, 24);
}

#[test]
fn across_mod_auto() {
    #[macro_use]
    mod mod_a {
        use yuuka::derive_struct;

        derive_struct!(
            #[derive(PartialEq)]
            pub Root { a: String, b: i32 }
        );

        #[macro_use]
        pub mod mod_b {
            use yuuka::derive_enum;

            derive_enum!(
                #[derive(PartialEq)]
                pub enum Root2 {
                    A,
                    B(i32),
                }
            );
        }
    }

    use yuuka::auto;

    use mod_a::mod_b::*;
    use mod_a::*;

    assert_eq!(
        auto!(Root {
            a: "hello".to_string(),
            b: 42
        }),
        Root {
            a: "hello".to_string(),
            b: 42
        }
    );
    assert_eq!(auto!(Root2::A), Root2::A);
}

#[macro_use]
mod across_mod_1 {
    use yuuka::derive_struct;

    derive_struct!(Root {
        a: {
            b: String
        }
    });
}

mod across_mod_2 {
    use yuuka::auto;

    use super::across_mod_1::*;

    #[test]
    fn test() {
        let val = auto!(Root {
            a: {
                b: "hello".to_string()
            }
        });
        assert_eq!(val.a.b, "hello");
    }
}
