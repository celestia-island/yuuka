# السمات والرؤية

يغطي هذا المستند كيفية التحكم بمشتقات الماكرو، وسمات الماكرو، والرؤية، والتصدير عبر الصناديق للأنواع المُولَّدة بواسطة `derive_struct!` و `derive_enum!`.

---

## مشتقات إضافية

ضع `#[derive(...)]` قبل اسم النوع لإضافة مشتقات الماكرو إلى النوع الجذر المُولَّد:

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

> **ملاحظة**: يتم اشتقاق `Debug` و `Clone` تلقائياً دائماً. لا تحتاج لتحديدهما.

ينطبق الأمر نفسه على `derive_enum!`:

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

## سمات الماكرو

ضع سمات الماكرو بعد `#[derive(...)]`:

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

## نشر السمات بشكل تكراري

استخدم `#[macros_recursive(...)]` لنشر السمات إلى **جميع** الأنواع المضمنة المتداخلة:

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
// جميع المستويات المتداخلة تستخدم camelCase: "nickName"، "simplifiedChinese"، "firstName"، إلخ.
```

`#[macros_recursive(...)]` يُطبّق السمات المحددة على كل هيكل وتعداد مُولَّد في التسلسل الهرمي — وليس فقط النوع الجذر.

---

## سمات مستوى الحقل

ضع السمات مباشرةً قبل اسم الحقل:

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

// "live_in" يُسلسل كـ "location" بدلاً من "liveIn"
```

### سمات مستوى المتغيرات للتعدادات

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

## سمات النوع على الأنواع المضمنة

يمكنك تطبيق `#[derive(...)]` والسمات على أنواع الهياكل/التعدادات المضمنة المُعرَّفة في حقل. ضعها **قبل اسم الحقل**، باستخدام `#[derive(...)]` لفصل سمات الحقل عن سمات النوع:

### أنواع مضمنة مُسمّاة

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

// Root يحصل على #[serde(deny_unknown_fields)]
// Location يحصل على #[derive(PartialEq)] و #[serde(rename_all = "UPPERCASE")]
// الحقل "location" يُعاد تسميته إلى "position"
```

### أنواع مضمنة مجهولة

للأنواع المجهولة، استخدم `#[derive]` (derive فارغ) كفاصل:

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

// #[derive] الفارغ يفصل سمات مستوى الحقل (فوقه) عن سمات مستوى النوع (تحته)
```

### على متغيرات التعداد

ينطبق نفس النمط على متغيرات صف التعداد:

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

ولمتغيرات التعداد المجهولة:

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

## الرؤية

### المُعدِّل `pub`

استخدم `pub` لجعل الأنواع المُولَّدة ووحدتها عامة:

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

يُولّد هذا `pub mod __Root` و `pub use __Root::*`، مما يجعل جميع الأنواع متاحة من خارج الوحدة الحالية.

### الرؤية الافتراضية

بدون `pub`، تكون الأنواع `pub(crate)`:

```rust
derive_struct!(
    Root {
        name: String,
    }
);
// يُولّد: pub(crate) mod __Root { ... }
// يُولّد: pub(crate) use __Root::*;
```

> **ملاحظة**: تُستخدم إعلانات `pub` عادةً على مستوى الوحدة أو الصندوق (خارج الدوال). داخل دوال الاختبار، لا تهم الرؤية.

---

## الاستخدام عبر الصناديق

لتصدير الأنواع المُولَّدة ووحدات ماكرو `auto!` المساعدة لاستخدامها في صناديق أخرى، استخدم `#[macro_export]`:

### صندوق المكتبة

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

> **ملاحظة**: يمكن وضع `#[macro_export]` قبل أو بعد `#[derive(...)]` — كلا الموضعين يعملان.

### الصندوق المستهلك

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

### كيف يعمل

`#[macro_export]` يجعل وحدات ماكرو `macro_rules!` المساعدة المُولَّدة (مثل `__auto_TestStruct!`) متاحة على مستوى جذر الصندوق. بدون هذه السمة، تكون وحدات الماكرو المساعدة مرئية فقط داخل الصندوق المُعرِّف، ولن يعمل `auto!` من صناديق خارجية.

### إعداد Cargo.toml

لصندوق المكتبة، تأكد من إمكانية ربطه بشكل صحيح:

```toml
[lib]
crate-type = ["rlib", "dylib"]
```

### مثال كامل

فيما يلي مثال كامل يجمع بين جميع الميزات — مشتقات إضافية، سمات تكرارية، سمات مستوى الحقل، الرؤية، والتصدير عبر الصناديق:

```rust
use serde::{Serialize, Deserialize};
use yuuka::{derive_struct, derive_enum, auto};

// تعريف هيكل مع جميع الميزات
derive_struct!(
    #[derive(PartialEq, Serialize, Deserialize)]
    #[macros_recursive(serde(rename_all = "camelCase"))]
    #[macro_export]
    pub AppConfig {
        app_name: String = "MyApp".to_string(),
        #[serde(rename = "ver")]
        version: String = "1.0.0".to_string(),
        server: {
            host: String = "localhost".to_string(),
            port: u16 = 8080,
        },
        features: [Feature {
            feature_name: String,
            enabled: bool = true,
        }],
        log_level: enum LogLevel {
            Debug,
            Info,
            Warn,
            Error,
        } = Info,
    }
);

let config = auto!(AppConfig {
    app_name: "Production".to_string(),
    server: {
        port: 443,
        ..Default::default()
    },
    features: vec![Feature {
        feature_name: "auth".to_string(),
        ..Default::default()
    }],
    ..Default::default()
});
```

يُظهر هذا المثال:

- **مشتقات إضافية**: `PartialEq`، `Serialize`، `Deserialize`
- **سمات تكرارية**: `serde(rename_all = "camelCase")` تُطبَّق على جميع الأنواع المتداخلة
- **سمات مستوى الحقل**: `#[serde(rename = "ver")]` على حقل `version`
- **الرؤية**: `pub` يجعل جميع الأنواع عامة
- **التصدير عبر الصناديق**: `#[macro_export]` يُتيح الاستخدام من صناديق أخرى
- **القيم الافتراضية**: تعبير الانتشار `..Default::default()` لملء الحقول المتبقية
