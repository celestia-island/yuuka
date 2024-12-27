use yuuka::derive_struct;

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
