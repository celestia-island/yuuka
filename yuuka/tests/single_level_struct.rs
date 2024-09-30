#[cfg(test)]
mod test {
    use yuuka::derive_config;

    #[test]
    fn single_level_struct() {
        derive_config!(Root { a: String, b: i32 });

        // let _ = Root {
        //     a: "Hello".to_string(),
        //     b: 42,
        // };
    }
}
