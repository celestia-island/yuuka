#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};
    use yuuka::derive_struct;

    #[test]
    fn extra_derive_struct() {
        derive_struct!(
            #[derive(Serialize, Deserialize)]
            #[serde(rename_all = "camelCase")]
            Root {
                nick_name: String,
                live_in: String,
            }
        );

        let ret = Root {
            nick_name: "langyo".to_string(),
            live_in: "China".to_string(),
        };
        assert_eq!(
            serde_json::to_string(&ret).unwrap(),
            r#"{"nickName":"langyo","liveIn":"China"}"#
        );
    }

    #[test]
    fn extra_derive_enum() {
        derive_struct!(
            #[derive(Serialize, Deserialize)]
            #[serde(rename_all = "snake_case")]
            Root {
                nick_name: enum {
                    SaibaMomoi,
                    SaibaMidori,
                    HanaokaYuzu,
                    TendouAris,
                } = SaibaMidori
            }
        );

        let ret = Root::default();
        assert_eq!(
            serde_json::to_string(&ret).unwrap(),
            r#"{"nick_name":"saiba_midori"}"#
        );
    }

    #[test]
    fn extra_derive_struct_with_multi_level() {
        derive_struct!(
            #[derive(Serialize, Deserialize)]
            #[serde(rename_all = "camelCase")]
            Root {
                nick_name: {
                    chinese: {
                        simplified_chinese: {
                            first_name: {
                                origin: String = "早濑".to_string(),
                                meme: String = "旱濑".to_string(),
                            },
                            last_name: String = "优香".to_string(),
                        },
                        traditional_chinese: {
                            first_name: String = "早瀨".to_string(),
                            last_name: String = "優香".to_string(),
                        },
                    }
                    japanese: {
                        first_name: String = "早瀬".to_string(),
                        last_name: String = "ユウカ".to_string(),
                    },
                    korean: {
                        first_name: String = "하야세".to_string(),
                        last_name: String = "유우카".to_string(),
                    },
                    english: {
                        first_name: String = "Hayase".to_string(),
                        last_name: String = "Yuuka".to_string(),
                    }
                },
            }
        );

        let ret = Root::default();
        assert_eq!(
            serde_json::to_string(&ret).unwrap(),
            r#"{"nickName":{"chinese":{"simplifiedChinese":{"firstName":{"origin":"早濑","meme":"旱濑"},"lastName":"优香"},"traditionalChinese":{"firstName":"早瀨","lastName":"優香"}},"japanese":{"firstName":"早瀬","lastName":"ユウカ"},"korean":{"firstName":"하야세","lastName":"유우카"},"english":{"firstName":"Hayase","lastName":"Yuuka"}}}"#
        );
    }

    #[test]
    fn extra_derive_struct_for_keys() {
        derive_struct!(
            #[derive(Serialize, Deserialize)]
            #[serde(rename_all = "camelCase")]
            Root {
                nick_name: String,
                #[serde(rename = "location")]
                live_in: String,
            }
        );

        let ret = Root {
            nick_name: "langyo".to_string(),
            live_in: "China".to_string(),
        };
        assert_eq!(
            serde_json::to_string(&ret).unwrap(),
            r#"{"nickName":"langyo","location":"China"}"#
        );
    }

    #[test]
    fn extra_derive_enum_for_keys() {
        derive_struct!(
            #[derive(Serialize, Deserialize)]
            #[serde(rename_all = "snake_case")]
            Root {
                nick_name: enum {
                    SaibaMomoi,
                    SaibaMidori,
                    #[serde(rename = "yuzu")]
                    HanaokaYuzu,
                    TendouAris,
                } = HanaokaYuzu
            }
        );

        let ret = Root::default();
        assert_eq!(
            serde_json::to_string(&ret).unwrap(),
            r#"{"nick_name":"yuzu"}"#
        );
    }
}
