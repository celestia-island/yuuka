#[cfg(test)]
mod test {
    use yuuka::derive_config;

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct C {
        d: f64,
    }

    #[test]
    fn complex_type_struct() {
        derive_config!(Root {
            a: String,
            b: i32,
            c: super::C,
        });

        let _ = Root {
            a: "Hello".to_string(),
            b: 42,
            c: C { d: 3.14 },
        };
    }
}
