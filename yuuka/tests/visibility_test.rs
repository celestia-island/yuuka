#[cfg(test)]
mod test {
    use yuuka::derive_struct;

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
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
}
