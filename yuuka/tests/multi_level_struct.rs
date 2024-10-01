#[cfg(test)]
mod test {
    use yuuka::derive_config;

    #[test]
    fn multi_level_struct() {
        derive_config!(Root {
            a: String,
            b: i32,
            c: C {
                d: f64,
                e: E { f: bool },
            },
        });

        // let _ = Root {
        //     a: "Hello".to_string(),
        //     b: 42,
        //     c: {
        //         d: 3.14,
        //         e: {
        //             f: true,
        //         },
        //     },
        // };
    }
}
