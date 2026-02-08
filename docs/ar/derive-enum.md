# ماكرو `derive_enum!`

يُعرِّف ماكرو `derive_enum!` أنواع تعدادات مستقلة بنفس أسلوب صياغة DSL المستخدم في `derive_struct!`. يدعم جميع أشكال المتغيرات الثلاثة، والأنواع المتداخلة، والقيم الافتراضية.

## الصياغة الأساسية

```rust
use yuuka::derive_enum;

derive_enum!(
    enum Status {
        Active,
        Inactive,
    }
);
```

يُولّد هذا تعداداً مع تطبيق `#[derive(Debug, Clone)]` تلقائياً.

---

## أشكال المتغيرات

### متغيرات الوحدة

متغيرات بسيطة بدون بيانات مرتبطة:

```rust
derive_enum!(
    enum Direction {
        North,
        South,
        East,
        West,
    }
);
```

### متغيرات شبيهة بالهيكل

متغيرات ذات حقول مُسمّاة. يمكن للحقول استخدام تعريفات هياكل مضمنة:

```rust
derive_enum!(
    enum Action {
        Move { x: f64, y: f64 },
        Attack {
            target: Target {
                id: u64,
                name: String,
            },
            damage: u32,
        },
    }
);
// يُولّد هيكل `Target` مستقل إلى جانب تعداد `Action`.
```

### متغيرات شبيهة بالصف

متغيرات ذات بيانات موضعية. يمكن أن تحتوي على هياكل مضمنة، تعدادات، وأنواع ثابتة:

```rust
derive_enum!(
    enum Message {
        Text(String),
        Data(Payload { content: Vec<u8>, size: usize }),
        Multi(String, i32, bool),
    }
);
```

### متغيرات مختلطة

يمكن أن تتواجد الأشكال الثلاثة معاً في تعداد واحد:

```rust
derive_enum!(
    #[derive(PartialEq, Serialize, Deserialize)]
    enum Router {
        Home,
        User { id: u64, name: String },
        Error(String),
    }
);
```

---

## التعدادات المتداخلة

يمكن أن تحتوي متغيرات التعداد على تعدادات مضمنة أخرى:

### في متغيرات الصف

```rust
derive_enum!(
    enum Group {
        Millennium(enum Millennium {
            GameDevelopment(enum GameDevelopment {
                Momoi,
                Midori,
                Yuzu,
                Arisu,
            }),
            CAndC,
            Veritas,
        }),
    }
);

let _ = Group::Millennium(Millennium::GameDevelopment(GameDevelopment::Yuzu));
```

### التعدادات المتداخلة المجهولة

```rust
derive_enum!(
    #[derive(PartialEq)]
    enum Root {
        A,
        B(i32),
        C { a: String, b: i32 },
        D(enum {
            E,
            F(i32),
            G { a: String, b: i32 },
        }),
    }
);
```

تُسمّى التعدادات المجهولة داخل المتغيرات مثل `_Root_0_anonymous`. يمكنك الإشارة إليها عبر الوحدة:

```rust
let _ = Root::D(__Root::_Root_0_anonymous::E);
```

> **نصيحة**: استخدم [`auto!`](./auto-macro.md) لتجنب التعامل مع الأسماء المجهولة المُولَّدة. `auto!(Root::D::E)` يحل المسار تلقائياً.

### تعدادات مجهولة متداخلة بعمق

```rust
derive_enum!(
    #[derive(PartialEq)]
    enum A {
        B(enum {
            C(enum {
                D(enum {
                    E(enum {
                        F,
                        G(String),
                    }),
                }),
            }),
        }),
    }
);

// الإنشاء اليدوي:
let _ = A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(_A_3_anonymous::F))));

// مع auto!:
use yuuka::auto;
let _ = auto!(A::B::C::D::E::F);
let _ = auto!(A::B::C::D::E::G("hello".to_string()));
```

---

## القيم الافتراضية

حدد متغيراً افتراضياً باستخدام `= VariantName` بعد القوس المغلق:

```rust
derive_enum!(
    enum Theme {
        Light,
        Dark,
        System,
    } = Dark
);

let theme = Theme::default();
// theme == Theme::Dark
```

### القيمة الافتراضية لمتغيرات الصف

```rust
derive_enum!(
    enum Value {
        Int(i32),
        Text(String),
    } = Int(0)
);
```

### القيمة الافتراضية للتعدادات المجهولة المتداخلة

```rust
derive_enum!(
    enum Group {
        Millennium(enum {
            GameDevelopment(enum GameDevelopment {
                Momoi,
                Midori,
                Yuzu,
                Arisu,
            } = Yuzu),
            CAndC,
            Veritas,
        } = GameDevelopment(Default::default())),
    } = Millennium(Default::default())
);

// Group::default() == Group::Millennium(GameDevelopment(Yuzu))
```

> **ملاحظة**: عند عدم تحديد قيمة افتراضية، يستخدم `impl Default` المُولَّد `unimplemented!()`، مما سيسبب حالة ذعر (panic) أثناء التشغيل إذا تم استدعاؤه. حدد دائماً قيمة افتراضية إذا كنت تنوي استخدام `Default::default()`.

---

## مشتقات وسمات إضافية

تماماً مثل `derive_struct!`، يمكنك تمرير `#[derive(...)]` وسمات الماكرو:

```rust
derive_enum!(
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Member {
        SaibaMomoi,
        SaibaMidori,
        HanaokaYuzu,
        TendouAris,
    } = SaibaMidori
);

let json = serde_json::to_string(&Member::default()).unwrap();
assert_eq!(json, r#""saiba_midori""#);
```

انظر [السمات والرؤية](./attributes.md) للتفاصيل الكاملة حول سمات الماكرو، والنشر التكراري، وسمات مستوى المتغيرات.

### التفاعل مع `derive_struct!`

يعمل `derive_enum!` بسلاسة مع `derive_struct!`. يمكن تعريف التعدادات مضمنةً داخل حقول الهيكل باستخدام `derive_struct!`، أو تعريفها بشكل مستقل باستخدام `derive_enum!` والإشارة إليها عبر المسار. كلا النهجين يولّدان أنواعاً متوافقة تماماً.

عند استخدام تعداد مُعرَّف عبر `derive_enum!` في هيكل `derive_struct!`، أشر إليه كنوع خارجي باستخدام بادئة `super::`:

```rust
derive_enum!(
    enum Status {
        Active,
        Inactive,
    }
);

derive_struct!(
    Root {
        name: String,
        status: super::Status,
    }
);
```
