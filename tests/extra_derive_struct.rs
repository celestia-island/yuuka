use serde::{Deserialize, Serialize};
use yuuka::{derive_enum, derive_struct};

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
    derive_enum!(
        #[derive(Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum Root {
            SaibaMomoi,
            SaibaMidori,
            HanaokaYuzu,
            TendouAris,
        } = SaibaMidori
    );

    let ret = Root::default();
    assert_eq!(serde_json::to_string(&ret).unwrap(), r#""saiba_midori""#);
}

#[test]
fn extra_derive_struct_with_multi_level() {
    derive_struct!(
        #[derive(Serialize, Deserialize)]
        #[macros_recursive(serde(rename_all = "camelCase"))]
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

#[test]
fn extra_derive_derive_for_typed_keys() {
    derive_struct!(
        #[derive(Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        Root {
            nick_name: String,
            #[serde(rename = "position")]
            #[derive(PartialEq)]
            #[serde(rename_all = "UPPERCASE")]
            location: Location {
                country: String,
                address: String,
            },
        }
    );

    assert_eq!(
        serde_json::to_string(&Root {
            nick_name: "arisu".to_string(),
            location: Location {
                country: "kivotos".to_string(),
                address: "777".to_string(),
            },
        })
        .unwrap(),
        r#"{"nick_name":"arisu","position":{"COUNTRY":"kivotos","ADDRESS":"777"}}"#
    );
}

#[test]
fn extra_derive_derive_for_anonymous_keys() {
    derive_struct!(
        #[derive(Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        Root {
            nick_name: String,
            #[serde(rename = "position")]
            #[derive]
            #[serde(rename_all = "UPPERCASE")]
            location: {
                country: String = "kivotos".to_string(),
                address: String = "777".to_string(),
            },
        }
    );

    assert_eq!(
        serde_json::to_string(&Root {
            nick_name: "arisu".to_string(),
            location: Default::default(),
        })
        .unwrap(),
        r#"{"nick_name":"arisu","position":{"COUNTRY":"kivotos","ADDRESS":"777"}}"#
    );
}

#[test]
fn extra_derive_enum_for_typed_keys() {
    derive_enum!(
        #[derive(Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Group {
            #[serde(rename = "777")]
            #[derive(PartialEq)]
            #[serde(rename_all = "UPPERCASE")]
            Millennium(enum Millennium {
                GameDevelopment(enum GameDevelopment {
                    Momoi,
                    Midori,
                    Yuzu,
                    Arisu,
                }),
                #[serde(rename = "C&C")]
                CAndC,
                Veritasu,
            })
        }
    );

    assert_eq!(
        serde_json::to_string(&Group::Millennium(Millennium::GameDevelopment(
            GameDevelopment::Yuzu
        )))
        .unwrap(),
        r#"{"777":{"GAMEDEVELOPMENT":"Yuzu"}}"#
    );
    assert_eq!(
        serde_json::to_string(&Group::Millennium(Millennium::CAndC)).unwrap(),
        r#"{"777":"C&C"}"#
    );
}

#[test]
fn extra_derive_enum_for_anonymous_keys() {
    derive_enum!(
        #[derive(Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Group {
            #[serde(rename = "777")]
            #[derive]
            #[serde(rename_all = "UPPERCASE")]
            Millennium(enum {
                GameDevelopment(enum GameDevelopment {
                    Momoi,
                    Midori,
                    Yuzu,
                    Arisu,
                } = Yuzu),
                #[serde(rename = "C&C")]
                CAndC,
                Veritasu,
            } = GameDevelopment(Default::default()))
        } = Millennium(Default::default())
    );

    assert_eq!(
        serde_json::to_string(&Group::Millennium(_Group_0_anonymous::GameDevelopment(
            GameDevelopment::Yuzu
        )))
        .unwrap(),
        r#"{"777":{"GAMEDEVELOPMENT":"Yuzu"}}"#
    );
    assert_eq!(
        serde_json::to_string(&Group::Millennium(_Group_0_anonymous::CAndC)).unwrap(),
        r#"{"777":"C&C"}"#
    );
}
