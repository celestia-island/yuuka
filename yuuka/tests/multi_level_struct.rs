#[cfg(test)]
mod test {
    use yuuka::derive_struct;

    #[test]
    fn multi_level_struct() {
        derive_struct!(Root {
            a: String,
            b: i32,
            c: C {
                d: f64,
                e: E { f: bool },
            },
        });

        let _ = Root {
            a: "Hello".to_string(),
            b: 42,
            c: C {
                d: 3.14,
                e: E { f: true },
            },
        };
    }
}
