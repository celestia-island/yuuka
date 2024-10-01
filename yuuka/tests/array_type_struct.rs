#[cfg(test)]
mod test {
    use yuuka::derive_config;

    #[test]
    fn array_type_struct() {
        derive_config!(Root {
            a: [A { b: String }]
        });

        let _ = Root {
            a: vec![A {
                b: "hello".to_string(),
            }],
        };
    }
}
