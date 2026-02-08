# 예제

실제 사용 예제와 yuuka 매크로가 생성하는 코드의 구조 설명입니다.

---

## 언어 팩 (i18n)

일반적인 사용 사례: JSON 파일에 직접 매핑되는 중첩 언어 팩 구조를 정의합니다. serde 를 사용한 직렬화/역직렬화를 지원합니다.

```rust
use anyhow::Result;
use serde::{Serialize, Deserialize};
use yuuka::{derive_struct, auto};

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
                },
            },
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
            },
        },
    });

    // JSON에서 역직렬화
    let json_raw = r#"
    {
        "是": "Yes", "否": "No", "确认": "Confirm",
        "取消": "Cancel", "保存": "Save",
        "主页": { "启动": "Start", "设置": "Settings" },
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
    }"#;

    let config_from_json = serde_json::from_str::<LanguagePack>(json_raw)?;
    assert_eq!(config, config_from_json);
    assert_eq!(config.设置.网络配置.代理地址, "Proxy Address");

    Ok(())
}
```

언어 팩 핵심 포인트:

- 필드 이름에 **비 ASCII 문자**(한자 등)를 사용할 수 있습니다 — Rust 식별자와 JSON 키 모두로 작동합니다.
- `auto!` 매크로는 익명 하위 구조체 구축(主页, 设置, 网络配置)을 원활하게 처리합니다.
- 생성된 타입은 JSON 왕복 직렬화를 위해 완전한 serde 호환성을 갖습니다.

---

## 서버 라우터 설정

더 복잡한 예제로, 중첩 배열과 인라인 열거형을 사용한 리버스 프록시/서버 라우터 설정을 모델링합니다.

```rust
use anyhow::Result;
use serde::{Serialize, Deserialize};
use yuuka::{derive_struct, auto};

fn main() -> Result<()> {
    derive_struct!(
        #[derive(PartialEq, Serialize, Deserialize)]
        Config {
            port: u16,
            services: [Service {
                domain: Vec<String>,
                rules: [Rule {
                    pattern: String,
                    method: enum Method {
                        Redirect { url: String },
                        Proxy { host: String },
                        StaticFile { path: String },
                        StaticDir { path: String },
                    },
                }],
            }],
        }
    );

    let config = auto!(Config {
        port: 8080,
        services: vec![Service {
            domain: vec!["example.com".to_string()],
            rules: vec![
                Rule {
                    pattern: "^/$".to_string(),
                    method: Method::Redirect {
                        url: "https://example.com/index.html".to_string(),
                    },
                },
                Rule {
                    pattern: "^/api".to_string(),
                    method: Method::Proxy {
                        host: "http://localhost:8081".to_string(),
                    },
                },
                Rule {
                    pattern: "^/static".to_string(),
                    method: Method::StaticDir {
                        path: "/var/www/static".to_string(),
                    },
                },
            ],
        }],
    });

    // 이 구조는 JSON과 직접적으로 변환 가능
    let json = serde_json::to_string_pretty(&config)?;
    let config_from_json: Config = serde_json::from_str(&json)?;
    assert_eq!(config, config_from_json);

    Ok(())
}
```

라우터 핵심 포인트:

- **`[Service { ... }]`** 는 `services: Vec<Service>` 를 생성하며, `Service` 를 독립적인 구조체로 만듭니다.
- **Service 내의 중첩된 `[Rule { ... }]`** 는 인라인 Rule 구조체와 함께 또 다른 `Vec<Rule>` 을 생성합니다.
- **`enum Method { ... }`** 는 열거형을 인라인으로 정의하며, 다양한 라우팅 방식에 대한 구조체형 배리언트를 갖습니다.
- 전체 설정을 JSON 으로 로드하거나 저장할 수 있습니다.

---

## 열거형을 사용한 앱 설정

열거형 기반 설정의 예제로, 다양한 배리언트 형태를 활용합니다:

```rust
use serde::{Serialize, Deserialize};
use yuuka::{derive_struct, derive_enum, auto};

derive_enum!(
    #[derive(PartialEq, Serialize, Deserialize)]
    enum Theme {
        Light,
        Dark,
        System,
    } = System
);

derive_struct!(
    #[derive(PartialEq, Serialize, Deserialize)]
    AppConfig {
        theme: super::Theme = super::Theme::System,
        language: String = "ko".to_string(),
        font_size: u32 = 14,
    }
);
```

설정 핵심 포인트:

- `derive_enum!` 으로 독립적인 열거형을 정의하여 `derive_struct!` 에서 참조할 수 있습니다.
- 열거형의 기본값(`= System`)은 `Default::default()` 호출 시 사용됩니다.
- 외부 타입을 참조할 때는 `super::` 접두사가 필요합니다 (생성된 타입이 모듈 내에 위치하므로).

---

## 생성된 코드 구조

yuuka 가 어떤 코드를 생성하는지 이해하면 라이브러리를 효과적으로 디버그하고 활용하는 데 도움이 됩니다.

다음과 같이 작성하면:

```rust
derive_struct!(
    #[derive(Serialize)]
    pub Root {
        name: String,
        child: Child {
            value: i32,
        },
    }
);
```

매크로는 대략 다음과 같은 코드를 생성합니다:

```rust
#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
pub mod __Root {
    use super::*;

    #[derive(Debug, Clone, Serialize, Default)]
    pub struct Root {
        pub name: String,
        pub child: Child,
    }

    #[derive(Debug, Clone, Serialize, Default)]
    pub struct Child {
        pub value: i32,
    }

    // auto!용 헬퍼 매크로
    macro_rules! __auto_Root {
        (name $($tt:tt)*) => { $($tt)* };
        (child { $($tt:tt)* }) => { ::yuuka::auto!(Child { $($tt)* }) };
        (child $($tt:tt)*) => { $($tt)* };
        // ... 각 필드에 대한 추가 규칙
    }

    macro_rules! __auto_Child {
        (value $($tt:tt)*) => { $($tt)* };
        // ... 각 필드에 대한 추가 규칙
    }
}
pub use __Root::*;
```

### 핵심 요소

1. **모듈 래핑**: 모든 타입은 이름 충돌을 방지하기 위해 `__TypeName` 이라는 모듈에 들어갑니다. 모든 것은 `use __TypeName::*` 로 재내보내기됩니다.

2. **자동 derive**: `Debug` 와 `Clone` 이 항상 추가됩니다. 사용자가 지정한 `#[derive(...)]` 매크로가 추가로 적용됩니다.

3. **Default 구현**: 커스텀 기본값이 있는 필드가 없으면 → `#[derive(Default)]`. 어떤 필드라도 `= value` 가 있으면 → 수동 `impl Default { ... }`.

4. **헬퍼 매크로**: 각 타입에 대해 `__auto_TypeName!` 매크로가 생성됩니다. 이것은 `auto!` 절차적 매크로가 필드 타입(특히 익명 구조체/열거형 이름)을 해석하기 위해 호출하는 `macro_rules!` 매크로입니다.

5. **super 임포트**: `use super::*` 가 외부 스코프를 모듈로 가져오므로, 외부 타입을 참조할 때 `super::` 접두사가 필요합니다.

### 모듈 명명 규칙

| 입력 | 모듈 이름 |
| --- | --- |
| `Root { ... }` | `__Root` |
| `Config { ... }` | `__Config` |
| Root 내 익명 필드 | `_Root_0_anonymous`, `_Root_1_anonymous`, ... |
| 열거형 A 내 익명 필드 | `_A_0_anonymous`, `_A_1_anonymous`, ... |
