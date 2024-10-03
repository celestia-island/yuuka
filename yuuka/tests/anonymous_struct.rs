#[cfg(test)]
mod test {
    use yuuka::{auto, derive_struct};

    #[test]
    fn derive_struct_anonymously() {
        derive_struct!(Root {
            a: [{ b: String }]
        });

        let _ = Root {
            a: vec![auto! {
                b: "hello".to_string(),
            }],
        };
    }
}
