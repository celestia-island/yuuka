use yuuka::derive_struct;

#[test]
fn option_type_basic() {
    derive_struct!(Root {
        a?: String
    });

    let _ = Root {
        a: Some("hello".to_string()),
    };
}

#[test]
fn option_type_struct() {
    derive_struct!(Root {
        a?: A { b: String }
    });

    let _ = Root {
        a: Some(A {
            b: "hello".to_string(),
        }),
    };
}

#[test]
fn option_type_struct_with_enum() {
    derive_struct!(Root {
        a?: A { b: String },
        b?: enum AttackType {
            Momoi,
            Midori,
            Yuzu,
            Arisu,
        },
    });

    let _ = Root {
        a: None,
        b: Some(AttackType::Momoi),
    };
}

#[test]
fn option_type_struct_anonymous() {
    derive_struct!(Root {
        a?: {
            b: String
        },
    });

    let _ = Root {
        a: Some(__Root::_Root_0_anonymous {
            b: "hello".to_string(),
        }),
    };
}

#[test]
fn option_type_enum_with_keys() {
    derive_struct!(Root {
        a?: enum AttackType {
            Momoi,
            Midori { b?: String },
            Yuzu,
            Arisu,
        },
    });

    let _ = Root {
        a: Some(AttackType::Midori {
            b: Some("hello".to_string()),
        }),
    };
}
