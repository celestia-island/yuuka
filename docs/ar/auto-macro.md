# ماكرو `auto!`

يُبسِّط ماكرو `auto!` إنشاء نسخ من الأنواع المُولَّدة بواسطة `derive_struct!` و `derive_enum!`. قيمته الأساسية هي حل أسماء الأنواع المجهولة تلقائياً — تكتب مسارات سهلة القراءة بينما يوسّعها الماكرو إلى الأسماء المُولَّدة الصحيحة.

## المفهوم الأساسي

عند استخدام هياكل أو تعدادات مجهولة، يُولّد yuuka أسماءً مثل `_Root_0_anonymous`. إنشاء هذه يدوياً مطوَّل وهش:

```rust
derive_struct!(Root {
    data: {
        name: String,
        score: f64,
    },
});

// بدون auto! — يجب أن تعرف الاسم المُولَّد
let val = Root {
    data: _Root_0_anonymous {
        name: "test".to_string(),
        score: 99.5,
    },
};

// مع auto! — فقط استخدم { }
let val = auto!(Root {
    data: {
        name: "test".to_string(),
        score: 99.5,
    },
});
```

---

## بناء الهياكل

### هيكل أساسي

```rust
derive_struct!(Root {
    a: String,
    b: i32,
});

let obj = auto!(Root {
    a: "hello".to_string(),
    b: 42,
});
```

### هياكل مجهولة متداخلة

```rust
derive_struct!(Root {
    a: String,
    b: i32,
    c: f64,
    d: {
        e: String = "world".to_string(),
        f: i32,
    },
});

let obj = auto!(Root {
    a: "hello".to_string(),
    b: 42,
    c: std::f64::consts::PI,
    d: {
        f: 24,
        ..Default::default()
    },
});
assert_eq!(obj.d.e, "world"); // من القيمة الافتراضية
assert_eq!(obj.d.f, 24);      // مُعيَّن صراحةً
```

### تعبير الانتشار

استخدم `..Default::default()` لملء الحقول المتبقية بالقيم الافتراضية، تماماً مثل صياغة تحديث الهيكل القياسية في Rust:

```rust
let obj = auto!(Root {
    a: "hello".to_string(),
    b: 42,
    c: 3.14,
    d: {
        f: 24,
        ..Default::default()  // e يحصل على القيمة الافتراضية "world"
    },
});
```

---

## بناء التعدادات

### متغير الوحدة

```rust
derive_enum!(
    #[derive(PartialEq)]
    enum Root {
        A,
        B(i32),
        C { a: String, b: i32 },
    }
);

assert_eq!(auto!(Root::A), Root::A);
```

### متغير الصف

```rust
assert_eq!(auto!(Root::B(42)), Root::B(42));
```

### متغير شبيه بالهيكل

```rust
assert_eq!(
    auto!(Root::C {
        a: "hello".to_string(),
        b: 42,
    }),
    Root::C {
        a: "hello".to_string(),
        b: 42,
    }
);
```

---

## حل مسار التعداد المجهول

هنا يتألق `auto!` حقاً. للتعدادات المجهولة المتداخلة داخل متغيرات الصف، يحل `auto!` المسار عبر مستويات متعددة:

### مستوى واحد

```rust
derive_enum!(
    #[derive(PartialEq)]
    enum Root {
        D(enum {
            E,
            F(i32),
            G { a: String, b: i32 },
        }),
    }
);

// بدون auto! — مطوَّل
let _ = Root::D(__Root::_Root_0_anonymous::E);

// مع auto! — نظيف
assert_eq!(auto!(Root::D::E), Root::D(__Root::_Root_0_anonymous::E));

assert_eq!(auto!(Root::D::F(42)), Root::D(__Root::_Root_0_anonymous::F(42)));

assert_eq!(
    auto!(Root::D::G {
        a: "hello".to_string(),
        b: 42,
    }),
    Root::D(__Root::_Root_0_anonymous::G {
        a: "hello".to_string(),
        b: 42,
    })
);
```

### مسارات متداخلة بعمق

يمكن لـ `auto!` حل المسارات عبر تداخل عشوائي العمق للتعدادات المجهولة:

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

// يحل: A::B → _A_0_anonymous::C → _A_1_anonymous::D → _A_2_anonymous::E → _A_3_anonymous::F
assert_eq!(
    auto!(A::B::C::D::E::F),
    A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(_A_3_anonymous::F))))
);

assert_eq!(
    auto!(A::B::C::D::E::G("hello".to_string())),
    A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(
        _A_3_anonymous::G("hello".to_string())
    ))))
);
```

---

## الاستخدام المختلط

يمكنك تداخل استدعاءات `auto!` داخل استدعاءات `auto!` أخرى أو إنشاء هياكل عادية:

```rust
derive_struct!(
    #[derive(PartialEq)]
    Root {
        outer: {
            a: enum B {
                C {
                    c: i32,
                    d: f64,
                },
            },
        },
    }
);

let val = auto!(Root {
    outer: {
        a: auto!(B::C { c: 42, d: std::f64::consts::PI }),
    },
});

assert_eq!(val.outer.a, B::C { c: 42, d: std::f64::consts::PI });
```

---

## الاستخدام عبر الوحدات

يعمل `auto!` عبر حدود الوحدات طالما أن الأنواع ووحدات الماكرو المساعدة في النطاق:

```rust
#[macro_use]
mod definitions {
    use yuuka::derive_struct;

    derive_struct!(
        #[derive(PartialEq)]
        pub Root {
            a: String,
            b: i32,
        }
    );
}

mod usage {
    use yuuka::auto;
    use super::definitions::*;

    #[test]
    fn test() {
        assert_eq!(
            auto!(Root {
                a: "hello".to_string(),
                b: 42,
            }),
            Root {
                a: "hello".to_string(),
                b: 42,
            }
        );
    }
}
```

عند استخدام أنواع مجهولة عبر الوحدات، تأكد من تعليم الوحدة المُعرِّفة بـ `#[macro_use]`:

```rust
#[macro_use]
mod definitions {
    use yuuka::derive_struct;

    derive_struct!(Root {
        data: {
            value: String,
        },
    });
}

mod usage {
    use yuuka::auto;
    use super::definitions::*;

    fn create() {
        let val = auto!(Root {
            data: {
                value: "hello".to_string(),
            },
        });
    }
}
```

للاستخدام عبر الصناديق، انظر [السمات والرؤية — الاستخدام عبر الصناديق](./attributes.md#الاستخدام-عبر-الصناديق).
