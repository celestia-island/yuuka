# Yuuka - مقدمة

**Yuuka** هي مكتبة وحدات ماكرو إجرائية (procedural macros) في Rust تتيح لك تعريف هياكل متداخلة معقدة وتسلسلات هرمية للتعدادات باستخدام صياغة DSL مختصرة تشبه JSON. وهي مبنية على `serde` للتسلسل وفك التسلسل بسلاسة.

## التثبيت

أضف ما يلي إلى ملف `Cargo.toml` الخاص بك:

```toml
[dependencies]
yuuka = "0.6"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

> `serde` و `serde_json` اختياريان لكنهما يُستخدمان عادةً مع yuuka لدعم التسلسل.

## وحدات الماكرو الأساسية

تصدّر yuuka ثلاث وحدات ماكرو إجرائية:

| الماكرو | الغرض |
| --- | --- |
| [`derive_struct!`](./derive-struct.md) | تعريف تسلسلات هرمية متداخلة للهياكل باستخدام DSL يشبه JSON |
| [`derive_enum!`](./derive-enum.md) | تعريف أنواع التعدادات بأشكال متنوعة من المتغيرات |
| [`auto!`](./auto-macro.md) | إنشاء نسخ من الأنواع المُولَّدة بواسطة الماكرو أعلاه بصياغة مبسطة |

انظر أيضاً:

- [السمات والرؤية](./attributes.md) — مشتقات إضافية، نشر السمات، التحكم بالرؤية، والاستخدام عبر الصناديق
- [أمثلة](./examples.md) — أمثلة واقعية وبنية الكود المُنشأ

## بداية سريعة

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

يقوم استدعاء `derive_struct!` هذا تلقائياً بتوليد ثلاث هياكل مستقلة — `GameConfig` و `Window` و `Plugin` — وجميعها مع `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`. ثم يتيح ماكرو `auto!` إنشاء نسخ باستخدام كتل `{ }` للهياكل الفرعية المجهولة/المضمنة دون الحاجة لمعرفة أسمائها المُولَّدة.

## فهرس الوثائق

| الوثيقة | الوصف |
| --- | --- |
| [derive_struct!](./derive-struct.md) | ماكرو تعريف الهياكل — هياكل متداخلة، هياكل مجهولة، أنواع Vec/Option، قيم افتراضية، تعدادات مضمنة، أنواع المراجع |
| [derive_enum!](./derive-enum.md) | ماكرو تعريف التعدادات — متغيرات الوحدة/الهيكل/الصف، تعدادات متداخلة، قيم افتراضية |
| [auto!](./auto-macro.md) | ماكرو إنشاء النسخ — صياغة مبسطة للأنواع المجهولة، مسارات التعدادات، تعبيرات الانتشار |
| [السمات والرؤية](./attributes.md) | مشتقات الماكرو، نشر السمات، `#[macros_recursive]`، سمات مستوى الحقل، الرؤية، `#[macro_export]`، الاستخدام عبر الصناديق |
| [أمثلة](./examples.md) | أمثلة واقعية، شرح بنية الكود المُنشأ |
