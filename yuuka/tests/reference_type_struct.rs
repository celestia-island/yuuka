#[cfg(test)]
mod test {
    use yuuka::derive_config;

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct C {
        d: f64,
    }

    #[test]
    fn reference_type_struct() {
        derive_config!(Root {
            a_b: String,
            B: i32,
            c: super::C,
        });

        let _ = Root {
            a_b: "Hello".to_string(),
            B: 42,
            c: C { d: 3.14 },
        };
    }
}
