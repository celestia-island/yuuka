use anyhow::Result;
use serde::{Deserialize, Serialize};
use yuuka::{auto, derive_struct};

fn main() -> Result<()> {
    derive_struct!(
        #[derive(PartialEq, Serialize, Deserialize)]
        LanguagePack {
          是: String,
          否: String,
          确认: String,
          取消: String,
          保存: String,
          主页: {
            启动: String,
            设置: String,
          },
          设置: {
            虚拟机路径: String,
            程序本体路径: String,
            网络配置: {
              网络配置: String,
              是否启用代理: String,
              代理地址: String,
              是否启用IPV6: String,
            }
          }
        }
    );

    let config = auto!(LanguagePack {
        是: "Yes".to_string(),
        否: "No".to_string(),
        确认: "Confirm".to_string(),
        取消: "Cancel".to_string(),
        保存: "Save".to_string(),
        主页: {
            启动: "Start".to_string(),
            设置: "Settings".to_string(),
        },
        设置: {
            虚拟机路径: "VM Path".to_string(),
            程序本体路径: "Program Path".to_string(),
            网络配置: {
                网络配置: "Network Config".to_string(),
                是否启用代理: "Enable Proxy".to_string(),
                代理地址: "Proxy Address".to_string(),
                是否启用IPV6: "Enable IPV6".to_string(),
            }
        }
    });

    let json_raw = r#"
{
    "是": "Yes",
    "否": "No",
    "确认": "Confirm",
    "取消": "Cancel",
    "保存": "Save",
    "主页": {
        "启动": "Start",
        "设置": "Settings"
    },
    "设置": {
        "虚拟机路径": "VM Path",
        "程序本体路径": "Program Path",
        "网络配置": {
            "网络配置": "Network Config",
            "是否启用代理": "Enable Proxy",
            "代理地址": "Proxy Address",
            "是否启用IPV6": "Enable IPV6"
        }
    }
}
        "#;

    let config_from_json = serde_json::from_str::<LanguagePack>(json_raw)?;
    assert_eq!(config, config_from_json);
    assert_eq!(config.设置.网络配置.代理地址, "Proxy Address");
    Ok(())
}
