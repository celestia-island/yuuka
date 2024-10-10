#[cfg(test)]
mod test {
    use yuuka::derive_struct;

    #[test]
    fn extra_derive_struct() {
        derive_struct!(
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
}
