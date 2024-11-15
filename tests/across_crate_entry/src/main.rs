use yuuka::auto;

use across_crate_lib::*;

fn main() {
    let test_struct = auto!(TestStruct {
        a: 1,
        b: "Hello".to_string(),
        c: {
            d: 2,
            e: "World".to_string(),
        }
    });
    assert_eq!(
        test_struct,
        TestStruct {
            a: 1,
            b: "Hello".to_string(),
            c: _TestStruct_0_anonymous {
                d: 2,
                e: "World".to_string(),
            }
        }
    );

    let test_enum = auto!(TestEnum::C::F::H("Hello".to_string()));
    assert_eq!(test_enum, TestEnum::C(C::F(F::H("Hello".to_string()))));
}
