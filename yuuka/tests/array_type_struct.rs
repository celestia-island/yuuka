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

    #[test]
    fn array_type_struct_with_enum() {
        derive_config!(Root {
            a: [A { b: String }],
            b: [enum AttackType {
                Momoi,
                Midori,
                Yuzu,
                Arisu,
            }],
        });

        let _ = Root {
            a: vec![A {
                b: "hello".to_string(),
            }],
            b: vec![
                AttackType::Momoi,
                AttackType::Midori,
                AttackType::Yuzu,
                AttackType::Arisu,
            ],
        };
    }
}
