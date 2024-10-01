#[cfg(test)]
mod test {
    use yuuka::derive_struct;

    #[test]
    fn enum_type_struct() {
        derive_struct!(Root {
            a: enum Member {
                Momoi,
                Midori,
                Yuzu,
                Arisu,
            },
        });

        let _ = Root { a: Member::Momoi };
    }

    #[test]
    fn enum_type_struct_with_braces() {
        derive_struct!(Root {
            a: enum Member {
                Momoi { skill: Skill {
                    name: String
                }},
                Midori { skills: Vec<String>, level: usize },
                Yuzu {
                    skill: Skill {
                        name: String
                    },
                    level: usize
                },
                Arisu { level: usize },
            },
        });

        let _ = Root {
            a: Member::Midori {
                skills: vec!["hello".to_string()],
                level: 1,
            },
        };
    }

    #[test]
    fn enum_type_struct_with_parentheses() {
        derive_struct!(Root {
            a: enum Member {
                Momoi (Skill {
                    name: String
                }),
                Midori (Vec<String>, usize),
                Yuzu (
                    Skill {
                        name: String
                    },
                    usize
                ),
                Arisu (usize),
            },
        });

        let _ = Root {
            a: Member::Midori(vec!["hello".to_string()], 1),
        };
    }

    #[test]
    fn enum_type_struct_with_enum_in_braces() {
        derive_struct!(Root {
            a: enum Member {
                Momoi,
                Midori,
                Yuzu,
                Arisu { ty: enum ArisuType {
                    Arisu,
                    Key
                } },
            },
        });

        let _ = Root {
            a: Member::Arisu { ty: ArisuType::Key },
        };
    }

    #[test]
    fn enum_type_struct_with_enum_in_parentheses() {
        derive_struct!(Root {
            a: enum Member {
                Momoi,
                Midori,
                Yuzu,
                Arisu(enum ArisuType {
                    Arisu,
                    Key
                }),
            },
        });

        let _ = Root {
            a: Member::Arisu(ArisuType::Key),
        };
    }
}
