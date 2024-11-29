use yuuka::{derive_enum, derive_struct};

#[derive(Debug, Clone, PartialEq, Default)]
struct C {
    d: f64,
}

derive_struct!(pub Root {
    a_b: String,
    B: i32,
    c: C,
});

#[test]
fn pub_type_struct() {
    let _ = Root {
        a_b: "Hello".to_string(),
        B: 42,
        c: C {
            d: std::f64::consts::PI,
        },
    };
}

derive_enum!(
    pub enum Root2 {
        A,
        B,
        C,
    }
);

#[test]
fn pub_type_enum() {
    let _ = Root2::A;
}
