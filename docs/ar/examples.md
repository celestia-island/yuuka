# أمثلة

أمثلة استخدام واقعية وشرح للكود المُولَّد بواسطة وحدات ماكرو yuuka.

---

## حزمة اللغة (i18n)

حالة استخدام نموذجية: تعريف بنية حزمة لغة متداخلة تتطابق مباشرةً مع ملفات JSON. تدعم التسلسل وفك التسلسل مع serde.

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

    // فك التسلسل من JSON
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

### النقاط الرئيسية لحزمة اللغة

- يمكن أن تكون أسماء الحقول **غير ASCII** (أحرف صينية، إلخ) — تعمل كمعرّفات Rust ومفاتيح JSON معاً.
- يتعامل ماكرو `auto!` مع إنشاء الهياكل الفرعية المجهولة (主页، 设置، 网络配置) بسلاسة.
- الأنواع المُولَّدة متوافقة تماماً مع serde للتسلسل ذهاباً وإياباً مع JSON.

---

## تكوين الموجّه

مثال أكثر تعقيداً يُصمّم تكوين وكيل عكسي / موجّه خادم مع مصفوفات متداخلة وتعدادات مضمنة.

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

    // تتطابق هذه البنية مباشرةً مع/من JSON
    let json = serde_json::to_string_pretty(&config)?;
    let config_from_json: Config = serde_json::from_str(&json)?;
    assert_eq!(config, config_from_json);

    Ok(())
}
```

### النقاط الرئيسية للموجّه

- **`[Service { ... }]`** يُولّد `services: Vec<Service>` مع `Service` كهيكل مستقل.
- **`[Rule { ... }]` المتداخل** داخل Service يُولّد `Vec<Rule>` آخر مع هيكل Rule مضمن.
- **`enum Method { ... }`** يُعرّف تعداداً مضمناً، مع متغيرات شبيهة بالهيكل لطرق التوجيه المختلفة.
- يمكن تحميل التكوين بالكامل من / حفظه إلى JSON.

---

## تكوين التطبيق مع التعدادات

مثال يجمع بين الهياكل والتعدادات لتكوين تطبيق واقعي:

```rust
use serde::{Serialize, Deserialize};
use yuuka::{derive_struct, auto};

derive_struct!(
    #[derive(PartialEq, Serialize, Deserialize)]
    AppConfig {
        name: String = "MyApp".to_string(),
        version: String = "1.0.0".to_string(),
        database: Database {
            host: String = "localhost".to_string(),
            port: u16 = 5432,
            driver: enum Driver {
                Postgres,
                MySQL,
                SQLite { path: String },
            } = Postgres,
        },
        logging: Logging {
            level: enum Level {
                Debug,
                Info,
                Warn,
                Error,
            } = Info,
            outputs: [enum Output {
                Console,
                File { path: String },
                Remote { url: String },
            }],
        },
    }
);

let config = auto!(AppConfig {
    database: {
        port: 3306,
        driver: Driver::MySQL,
        ..Default::default()
    },
    logging: {
        outputs: vec![
            Output::Console,
            Output::File { path: "/var/log/app.log".to_string() },
        ],
        ..Default::default()
    },
    ..Default::default()
});
```

### النقاط الرئيسية للتكوين

- **التعدادات المضمنة** (`Driver`، `Level`، `Output`) تُعرّف مباشرةً داخل حقول الهيكل.
- **القيم الافتراضية** تُبسّط الإنشاء — فقط حدد ما يختلف عن الافتراضي.
- **تعداد المصفوفة** (`[enum Output { ... }]`) يُولّد `Vec<Output>` للنتائج المتعددة.
- تعبير الانتشار `..Default::default()` يملأ الحقول المتبقية تلقائياً.

---

## هيكل الكود المُنشأ

فهم ما يُولّده yuuka يساعد في تصحيح الأخطاء والعمل بفعالية مع المكتبة.

عندما تكتب:

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

يُولّد الماكرو تقريباً:

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

    // وحدات ماكرو مساعدة لـ auto!
    macro_rules! __auto_Root {
        (name $($tt:tt)*) => { $($tt)* };
        (child { $($tt:tt)* }) => { ::yuuka::auto!(Child { $($tt)* }) };
        (child $($tt:tt)*) => { $($tt)* };
        // ... قواعد أخرى لكل حقل
    }

    macro_rules! __auto_Child {
        (value $($tt:tt)*) => { $($tt)* };
        // ... قواعد أخرى لكل حقل
    }
}
pub use __Root::*;
```

### الجوانب الرئيسية

1. **تغليف الوحدة**: جميع الأنواع تُوضع في وحدة باسم `__TypeName` لتجنب تعارض الأسماء. يتم إعادة تصدير كل شيء بـ `use __TypeName::*`.

2. **المشتقات التلقائية**: يُضاف `Debug` و `Clone` دائماً. مشتقات `#[derive(...)]` المخصصة تُلحق بها.

3. **تنفيذ Default**: إذا لم تكن هناك حقول بقيم افتراضية مخصصة → `#[derive(Default)]`. إذا كان أي حقل يحتوي على `= value` → `impl Default { ... }` يدوي.

4. **وحدات ماكرو مساعدة**: لكل نوع، يتم توليد ماكرو `__auto_TypeName!`. هذه وحدات ماكرو `macro_rules!` يستدعيها ماكرو `auto!` الإجرائي لحل أنواع الحقول — خاصةً أسماء الهياكل/التعدادات المجهولة.

5. **استيراد Super**: `use super::*` يُدخل النطاق الخارجي إلى الوحدة، ولهذا تحتاج الأنواع الخارجية إلى بادئة `super::` عند الإشارة إليها.

### اصطلاح تسمية الوحدات

| المدخل | اسم الوحدة |
| --- | --- |
| `Root { ... }` | `__Root` |
| `Config { ... }` | `__Config` |
| حقل مجهول في Root | `_Root_0_anonymous`، `_Root_1_anonymous`، ... |
| حقل مجهول في تعداد A | `_A_0_anonymous`، `_A_1_anonymous`، ... |
