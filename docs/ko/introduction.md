# Yuuka - 소개

**Yuuka** 는 간결한 JSON 스타일 DSL 구문을 사용하여 복잡하고 깊게 중첩된 구조체(struct) 및 열거형(enum) 계층 구조를 정의할 수 있는 Rust 절차적 매크로 라이브러리입니다. 원활한 직렬화 및 역직렬화를 위해 `serde` 위에 구축되었습니다.

## 설치

`Cargo.toml` 에 다음을 추가하세요:

```toml
[dependencies]
yuuka = "0.6"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

> `serde` 와 `serde_json` 은 선택 사항이지만, 직렬화 지원을 위해 yuuka 와 함께 자주 사용됩니다.

## 핵심 매크로

Yuuka 는 세 가지 절차적 매크로를 제공합니다:

| 매크로 | 용도 |
| --- | --- |
| [`derive_struct!`](./derive-struct.md) | JSON 스타일 DSL 로 중첩 구조체 계층 구조를 정의 |
| [`derive_enum!`](./derive-enum.md) | 다양한 배리언트 형태의 열거형 타입을 정의 |
| [`auto!`](./auto-macro.md) | 위 매크로로 생성된 타입의 인스턴스를 간소화된 구문으로 구축 |

참고 항목:

- [속성 및 가시성](./attributes.md) — 추가 derive 매크로, 속성 전파, 가시성 제어, 크레이트 간 사용
- [예제](./examples.md) — 실제 사용 예제 및 생성된 코드 구조

## 빠른 시작

```rust
use serde::{Serialize, Deserialize};
use yuuka::{derive_struct, auto};

derive_struct!(
    #[derive(PartialEq, Serialize, Deserialize)]
    GameConfig {
        title: String,
        window: Window {
            width: u32,
            height: u32,
            fullscreen: bool,
        },
        plugins: [Plugin {
            name: String,
            enabled: bool,
        }],
    }
);

let config = auto!(GameConfig {
    title: "My Game".to_string(),
    window: {
        width: 1920,
        height: 1080,
        fullscreen: true,
    },
    plugins: vec![
        Plugin {
            name: "Audio".to_string(),
            enabled: true,
        },
    ],
});
```

이 단일 `derive_struct!` 호출은 `GameConfig`, `Window`, `Plugin` 이라는 세 개의 독립적인 구조체를 자동으로 생성하며, 모두 `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]` 가 적용됩니다. `auto!` 매크로는 익명/인라인 하위 구조체의 생성된 이름을 몰라도 `{ }` 블록을 사용하여 인스턴스를 구축할 수 있게 해줍니다.

## 문서 색인

| 문서 | 설명 |
| --- | --- |
| [derive_struct!](./derive-struct.md) | 구조체 정의 매크로 — 중첩 구조체, 익명 구조체, Vec/Option 타입, 기본값, 인라인 열거형, 참조 타입 |
| [derive_enum!](./derive-enum.md) | 열거형 정의 매크로 — unit/struct/tuple 배리언트, 중첩 열거형, 기본값 |
| [auto!](./auto-macro.md) | 인스턴스 구축 매크로 — 익명 타입을 위한 간소화된 구문, 열거형 경로, 전개 표현식 |
| [속성 및 가시성](./attributes.md) | derive 매크로, 속성 전파, `#[macros_recursive]`, 필드 수준 속성, 가시성, `#[macro_export]`, 크레이트 간 사용 |
| [예제](./examples.md) | 실제 사용 예제, 생성된 코드 구조 설명 |
