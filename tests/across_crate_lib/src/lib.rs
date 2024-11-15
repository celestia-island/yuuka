use yuuka::{derive_enum, derive_struct};

derive_struct!(
    #[derive(PartialEq)]
    #[macro_export] // After the derive macros.
    pub TestStruct {
        a: i32,
        b: String,
        c: {
            d: i32,
            e: String,
        }
    }
);

derive_enum!(
    #[macro_export] // Behind the derive macros.
    #[derive(PartialEq)]
    pub enum TestEnum {
        A(i32),
        B(String),
        C(enum C {
            D(i32),
            E(String),
            F(enum F {
                G(i32),
                H(String),
            }),
        })
    }
);
