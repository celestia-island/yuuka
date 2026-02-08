# 속성 및 가시성

이 문서에서는 `derive_struct!` 과 `derive_enum!` 으로 생성된 타입에 대해 derive 매크로, 속성 매크로, 가시성, 크레이트 간 내보내기를 제어하는 방법을 다룹니다.

---

## 추가 derive 매크로

타입 이름 앞에 `#[derive(...)]` 를 배치하여 생성된 루트 타입에 derive 매크로를 추가합니다:

```rust
use serde::{Serialize, Deserialize};
use yuuka::derive_struct;

derive_struct!(
    #[derive(Serialize, Deserialize)]
    Root {
        name: String,
        value: i32,
    }
);
```

> **참고**: `Debug` 와 `Clone` 은 항상 자동으로 derive 됩니다. 별도로 지정할 필요가 없습니다.

`derive_enum!` 에서도 동일하게 작동합니다:

```rust
use yuuka::derive_enum;

derive_enum!(
    #[derive(Serialize, Deserialize)]
    enum Status {
        Active,
        Inactive,
    }
);
```

---

## 속성 매크로

속성 매크로는 `#[derive(...)]` 뒤에 배치합니다:

```rust
derive_struct!(
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    Root {
        user_name: String,
        home_dir: String,
    }
);

let json = serde_json::to_string(&Root {
    user_name: "langyo".to_string(),
    home_dir: "/home/langyo".to_string(),
}).unwrap();
assert_eq!(json, r#"{"userName":"langyo","homeDir":"/home/langyo"}"#);
```

---

## 재귀적 속성 전파

`#[macros_recursive(...)]` 를 사용하여 **모든** 중첩 인라인 타입에 속성을 전파합니다:

```rust
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
            },
            japanese: {
                first_name: String = "早瀬".to_string(),
                last_name: String = "ユウカ".to_string(),
            },
        },
    }
);

let json = serde_json::to_string(&Root::default()).unwrap();
// 모든 중첩 수준에서 camelCase 적용: "nickName", "simplifiedChinese", "firstName" 등
```

`#[macros_recursive(...)]` 는 루트 타입뿐만 아니라 계층 구조에서 생성된 모든 구조체와 열거형에 지정된 속성을 적용합니다.

---

## 필드 수준 속성

필드 이름 바로 앞에 속성을 배치합니다:

```rust
derive_struct!(
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    Root {
        nick_name: String,
        #[serde(rename = "location")]
        live_in: String,
    }
);

// "live_in"은 "liveIn" 대신 "location"으로 직렬화됨
```

### 열거형의 배리언트 수준 속성

```rust
derive_enum!(
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Member {
        SaibaMomoi,
        SaibaMidori,
        #[serde(rename = "yuzu")]
        HanaokaYuzu,
        TendouAris,
    } = HanaokaYuzu
);

let json = serde_json::to_string(&Member::default()).unwrap();
assert_eq!(json, r#""yuzu""#);
```

---

## 인라인 타입의 타입 수준 속성

필드에 정의된 인라인 구조체/열거형 타입에 `#[derive(...)]` 와 속성을 적용할 수 있습니다. 필드 속성과 타입 속성을 구분하기 위해 `#[derive(...)]` 를 **필드 이름 앞에** 배치합니다:

### 이름 있는 인라인 타입

```rust
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

// Root에는 #[serde(deny_unknown_fields)]가 적용됨
// Location에는 #[derive(PartialEq)]와 #[serde(rename_all = "UPPERCASE")]가 적용됨
// 필드 "location"은 "position"으로 이름 변경됨
```

### 익명 인라인 타입

익명 타입의 경우, `#[derive]` (빈 derive)를 구분자로 사용합니다:

```rust
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

// 빈 #[derive]가 필드 수준 속성(위)과 타입 수준 속성(아래)을 구분합니다
```

### 열거형 배리언트에서

동일한 패턴이 열거형 튜플 배리언트에서도 작동합니다:

```rust
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
            Veritas,
        }),
    }
);
```

익명 열거형 배리언트의 경우:

```rust
derive_enum!(
    #[derive(Serialize, Deserialize)]
    enum Group {
        #[serde(rename = "777")]
        #[derive]
        #[serde(rename_all = "UPPERCASE")]
        Millennium(enum {
            GameDevelopment(enum GameDevelopment {
                Momoi, Midori, Yuzu, Arisu,
            } = Yuzu),
            CAndC,
        } = GameDevelopment(Default::default())),
    } = Millennium(Default::default())
);
```

---

## 가시성

### `pub` 수정자

`pub` 를 사용하여 생성된 타입과 해당 모듈을 공개합니다:

```rust
derive_struct!(
    pub Root {
        name: String,
    }
);

derive_enum!(
    pub enum Status {
        Active,
        Inactive,
    }
);
```

이 코드는 `pub mod __Root` 와 `pub use __Root::*` 를 생성하여, 현재 모듈 외부에서 모든 타입에 접근할 수 있게 합니다.

### 기본 가시성

`pub` 없이는 타입이 `pub(crate)` 로 설정됩니다:

```rust
derive_struct!(
    Root {
        name: String,
    }
);
// 생성: pub(crate) mod __Root { ... }
// 생성: pub(crate) use __Root::*;
```

> **참고**: `pub` 선언은 일반적으로 모듈이나 크레이트 수준(함수 외부)에서 사용됩니다. 테스트 함수 내에서는 가시성이 중요하지 않습니다.

---

## 크레이트 간 사용

다른 크레이트에서 사용할 수 있도록 생성된 타입과 `auto!` 헬퍼 매크로를 내보내려면 `#[macro_export]` 를 사용합니다:

### 라이브러리 크레이트

```rust
use yuuka::{derive_struct, derive_enum};

derive_struct!(
    #[derive(PartialEq)]
    #[macro_export]
    pub TestStruct {
        a: i32,
        b: String,
        c: {
            d: i32,
            e: String,
        },
    }
);

derive_enum!(
    #[macro_export]
    #[derive(PartialEq)]
    pub enum TestEnum {
        A(i32),
        B(String),
        C(enum C {
            D(i32),
            E(String),
            F(enum F {
                G(i32),
                H(String),
            }),
        }),
    }
);
```

> **참고**: `#[macro_export]`는 `#[derive(...)]` 앞이나 뒤에 배치할 수 있습니다 — 두 위치 모두 작동합니다.

### 소비 크레이트

```rust
use yuuka::auto;
use my_lib::*;

let test_struct = auto!(TestStruct {
    a: 1,
    b: "Hello".to_string(),
    c: {
        d: 2,
        e: "World".to_string(),
    },
});

let test_enum = auto!(TestEnum::C::F::H("Hello".to_string()));
assert_eq!(test_enum, TestEnum::C(C::F(F::H("Hello".to_string()))));
```

### 작동 원리

`#[macro_export]` 는 생성된 `macro_rules!` 헬퍼 매크로(예: `__auto_TestStruct!`)를 크레이트 루트 수준에서 사용할 수 있게 합니다. 이 속성이 없으면 헬퍼 매크로는 정의 크레이트 내에서만 보이며, 외부 크레이트에서 `auto!` 가 작동하지 않습니다.

### Cargo.toml 설정

라이브러리 크레이트의 경우, 올바르게 링크될 수 있도록 합니다:

```toml
[lib]
crate-type = ["rlib", "dylib"]
```

---

## 완전한 예제

다음은 여러 속성 기능을 함께 사용하는 종합적인 예제입니다:

```rust
use serde::{Serialize, Deserialize};
use yuuka::{derive_struct, derive_enum, auto};

derive_struct!(
    #[derive(PartialEq, Serialize, Deserialize)]
    #[macros_recursive(serde(rename_all = "camelCase"))]
    pub Config {
        app_name: String = "MyApp".to_string(),
        #[serde(rename = "ver")]
        app_version: String = "1.0.0".to_string(),
        settings: {
            dark_mode: bool = false,
            font_size: u32 = 14,
        },
    }
);

let config = Config::default();
let json = serde_json::to_string(&config).unwrap();
// camelCase 적용 + 필드 수준의 rename 적용
```

이 예제는 다음을 보여줍니다:

- `#[derive(...)]` 를 통한 추가 derive 매크로
- `#[macros_recursive(...)]` 를 통한 재귀적 속성 전파
- `#[serde(rename = ...)]` 를 통한 필드 수준 속성
- `pub` 를 통한 가시성 제어
- 기본값이 있는 중첩 익명 구조체
