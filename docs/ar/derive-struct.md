# ماكرو `derive_struct!`

ماكرو `derive_struct!` هو جوهر yuuka. يتيح لك تعريف تسلسلات هرمية معقدة من الهياكل المتداخلة باستخدام DSL مختصر يشبه JSON. يتم استخراج جميع الأنواع المضمنة تلقائياً إلى تعريفات هياكل/تعدادات مستقلة من المستوى الأعلى.

## الصياغة الأساسية

```rust
use yuuka::derive_struct;

derive_struct!(
    Root {
        field1: String,
        field2: i32,
    }
);
```

يُولّد هذا هيكلاً مع تطبيق `#[derive(Debug, Clone, Default)]` تلقائياً:

```rust
#[derive(Debug, Clone, Default)]
pub(crate) struct Root {
    pub field1: String,
    pub field2: i32,
}
```

---

## الهياكل المتداخلة

عرّف هياكل فرعية مضمنة مباشرةً داخل الهيكل الأب بتحديد `FieldName { ... }` كنوع:

```rust
derive_struct!(
    Root {
        info: Info {
            name: String,
            detail: Detail {
                level: u32,
                score: f64,
            },
        },
    }
);
```

يُولّد هذا **ثلاث** هياكل مستقلة: `Root` و `Info` و `Detail`. كل منها هيكل Rust عادي مع جميع الحقول عامة ضمن الوحدة المُولَّدة.

يمكنك التداخل إلى عمق عشوائي:

```rust
derive_struct!(
    Root {
        a: A {
            b: B {
                c: C {
                    d: D {
                        value: String,
                    },
                },
            },
        },
    }
);
```

---

## الهياكل المجهولة

احذف اسم النوع لإنشاء هياكل ذات أسماء تلقائية:

```rust
derive_struct!(
    Root {
        data: {
            value: String,
        },
    }
);
```

يُسمّى الهيكل المجهول تلقائياً `_Root_0_anonymous`. عند وجود عدة أنواع مجهولة، تُرقَّم تسلسلياً:

```rust
derive_struct!(
    Root {
        a: {
            b: String,
        },
        c: {
            d: f64,
        },
    }
);
// يُولّد: _Root_0_anonymous (لـ a)، _Root_1_anonymous (لـ c)
```

يمكن أن تكون الهياكل المجهولة متداخلة بعمق:

```rust
derive_struct!(
    Root {
        a: {
            b: String,
            c: {
                d: f64 = std::f64::consts::PI,
                e: {
                    f: bool = false,
                },
            },
            g: {
                h: i32 = -114514,
            },
        },
    }
);

let root = Root::default();
assert_eq!(root.a.c.d, std::f64::consts::PI);
assert!(!root.a.c.e.f);
assert_eq!(root.a.g.h, -114514);
```

> **نصيحة**: استخدم ماكرو [`auto!`](./auto-macro.md) لإنشاء نسخ من الهياكل المجهولة دون الحاجة لمعرفة الأسماء المُولَّدة.

---

## أنواع المصفوفات (Vec)

استخدم صياغة `[Type { ... }]` لتعريف حقول `Vec<Type>`:

```rust
derive_struct!(
    Root {
        items: [Item {
            name: String,
            count: u32,
        }],
    }
);
// يُولّد الحقل: items: Vec<Item>
```

### عناصر مصفوفة مجهولة

```rust
derive_struct!(
    Root {
        items: [{
            name: String,
        }],
    }
);
// يُولّد: items: Vec<_Root_0_anonymous>
```

### مصفوفات التعدادات

```rust
derive_struct!(
    Root {
        statuses: [enum Status {
            Active,
            Inactive,
        }],
    }
);
// يُولّد: statuses: Vec<Status>
```

### مصفوفات تعدادات مجهولة

```rust
derive_struct!(
    Root {
        values: [enum {
            Momoi,
            Midori,
            Yuzu,
            Arisu,
        }],
    }
);
// يُولّد: values: Vec<_Root_0_anonymous>
```

---

## الأنواع الاختيارية (Option)

ألحق `?` باسم الحقل لتغليف النوع في `Option<T>`:

```rust
derive_struct!(
    Root {
        required: String,
        optional?: String,
    }
);
// يُولّد:
//   required: String,
//   optional: Option<String>,
```

### Option مع هيكل مضمن

```rust
derive_struct!(
    Root {
        detail?: Detail {
            info: String,
        },
    }
);
// يُولّد: detail: Option<Detail>
```

### Option مع هيكل مجهول

```rust
derive_struct!(
    Root {
        data?: {
            value: String,
        },
    }
);
// يُولّد: data: Option<_Root_0_anonymous>
```

### Option مع تعداد مضمن

```rust
derive_struct!(
    Root {
        status?: enum Status {
            Active,
            Inactive,
        },
    }
);
// يُولّد: status: Option<Status>
```

### Option داخل متغيرات التعداد

تعمل صياغة `?` أيضاً داخل متغيرات التعداد ذات الهياكل:

```rust
derive_struct!(
    Root {
        action?: enum Action {
            Midori { detail?: String },
        },
    }
);
```

---

## القيم الافتراضية

عيّن قيماً افتراضية باستخدام `=` بعد النوع:

```rust
derive_struct!(
    Config {
        host: String = "localhost".to_string(),
        port: u16 = 8080,
        debug: bool = false,
    }
);

let config = Config::default();
assert_eq!(config.host, "localhost");
assert_eq!(config.port, 8080);
assert_eq!(config.debug, false);
```

### السلوك

- الحقول **التي تحتوي** على `= value` صريح تستخدم تلك القيمة في `impl Default` المُولَّد.
- الحقول **بدون** `= value` تستخدم `Default::default()` (مثلاً `0` للأرقام، `""` للسلاسل النصية، `false` للقيم المنطقية).
- إذا كان **أي** حقل يحتوي على قيمة افتراضية مخصصة، يُولّد الماكرو كتلة `impl Default` يدوية بدلاً من `#[derive(Default)]`.

### القيم الافتراضية للمصفوفات

```rust
derive_struct!(
    Root {
        // فارغة افتراضياً
        items: [Item {
            name: String = "unnamed".to_string(),
        }],
    }
);

let root = Root::default();
assert_eq!(root.items.len(), 0); // Vec فارغ افتراضياً
```

مع قيمة افتراضية صريحة للمصفوفة:

```rust
derive_struct!(
    Root {
        items: [Item {
            name: String = "unnamed".to_string(),
        }] = vec![Item { name: "first".to_string() }],
    }
);

let mut root = Root::default();
assert_eq!(root.items.len(), 1);
assert_eq!(root.items[0].name, "first");

// العناصر الجديدة تحصل على القيم الافتراضية لمستوى Item
root.items.push(Default::default());
assert_eq!(root.items[1].name, "unnamed");
```

### القيم الافتراضية للتعدادات

```rust
derive_struct!(
    #[derive(PartialEq)]
    Root {
        member: enum Member {
            Momoi,
            Midori,
            Yuzu,
            Arisu,
        } = Midori,
    }
);

let root = Root::default();
assert_eq!(root.member, Member::Midori);
```

مصفوفة تعدادات مع قيم افتراضية:

```rust
derive_struct!(
    #[derive(PartialEq)]
    Root {
        members: [enum Member {
            Momoi,
            Midori,
            Yuzu,
            Arisu,
        } = Midori] = vec![Member::Arisu],
    }
);

let mut root = Root::default();
assert_eq!(root.members[0], Member::Arisu); // من القيمة الافتراضية لـ Vec
root.members.push(Default::default());
assert_eq!(root.members[1], Member::Midori); // من القيمة الافتراضية للتعداد
```

---

## التعدادات المضمنة

عرّف التعدادات مضمنةً داخل حقول الهيكل:

```rust
derive_struct!(
    Root {
        status: enum Status {
            Active,
            Inactive,
        },
    }
);
```

### أشكال المتغيرات

تدعم متغيرات التعداد ثلاثة أشكال:

**متغيرات الوحدة** — بدون بيانات مرتبطة:

```rust
a: enum Member {
    Momoi,
    Midori,
    Yuzu,
    Arisu,
}
```

**متغيرات شبيهة بالهيكل** — حقول مُسمّاة، يمكن أن تحتوي بدورها على هياكل وتعدادات مضمنة:

```rust
a: enum Member {
    Momoi {
        skill: Skill {
            name: String,
        },
    },
    Midori { skills: Vec<String>, level: usize },
    Yuzu {
        skill: SkillYuzu {
            name: String,
        },
        level: usize,
    },
    Arisu { level: usize },
}
```

**متغيرات شبيهة بالصف** — بيانات موضعية، تدعم الهياكل المضمنة، حقول متعددة، وأنواع ثابتة:

```rust
a: enum Member {
    Momoi(Skill { name: String }),
    Midori(Vec<String>, usize),
    Yuzu(SkillYuzu { name: String }, usize),
    Arisu(usize),
}
```

### التعدادات المتداخلة

يمكن تداخل التعدادات داخل متغيرات التعداد:

```rust
// تعداد في متغير شبيه بالهيكل
derive_struct!(
    Root {
        a: enum Member {
            Arisu {
                ty: enum ArisuType {
                    Arisu,
                    Key,
                },
            },
        },
    }
);
let _ = Root { a: Member::Arisu { ty: ArisuType::Key } };

// تعداد في متغير شبيه بالصف
derive_struct!(
    Root {
        a: enum Member {
            Arisu(enum ArisuType {
                Arisu,
                Key,
            }),
        },
    }
);
let _ = Root { a: Member::Arisu(ArisuType::Key) };
```

### مصفوفات التعدادات في المتغيرات

```rust
// Vec<enum> في متغير شبيه بالهيكل
derive_struct!(
    Root {
        a: enum Member {
            Arisu {
                ty: [enum ArisuType {
                    Arisu,
                    Key,
                }],
            },
        },
    }
);

// Vec<enum> في متغير شبيه بالصف
derive_struct!(
    Root {
        a: enum Member {
            Arisu([enum ArisuType {
                Arisu,
                Key,
            }]),
        },
    }
);
```

---

## أنواع المراجع

استخدم صياغة المسار للإشارة إلى أنواع مُعرَّفة خارجياً:

```rust
#[derive(Debug, Clone, PartialEq, Default)]
struct ExternalType {
    data: f64,
}

derive_struct!(
    Root {
        name: String,
        external: super::ExternalType,
    }
);
```

> **ملاحظة**: نظراً لأن الأنواع المُولَّدة تعيش داخل وحدة (`__Root`)، ستحتاج عادةً إلى `super::` للإشارة إلى أنواع من النطاق الخارجي. يعتمد المسار الدقيق على مكان تعريف النوع الخارجي نسبةً إلى استدعاء الماكرو.

أسماء الحقول مرنة — تعمل أسماء snake_case و PascalCase معاً:

```rust
derive_struct!(
    Root {
        a_b: String,
        B: i32,
        c: super::C,
    }
);
```
