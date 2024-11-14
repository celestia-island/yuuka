#[cfg(test)]
mod test {
    use yuuka::{derive_enum, derive_struct};

    #[test]
    fn single_level_struct() {
        derive_struct!(Root { a: String, b: i32 });

        let _ = Root {
            a: "Hello".to_string(),
            b: 42,
        };
    }

    #[test]
    fn single_level_enum() {
        derive_enum!(
            enum Root {
                A,
                B,
            }
        );

        let _ = Root::A;
    }
}
