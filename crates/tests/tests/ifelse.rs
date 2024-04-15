use std::collections::BTreeMap;

use pytohdl::{translate, PyContext};
use tohdl_tests::mod_10_str;

fn if_no_else_str() -> &'static str {
    r#"
def mod_10(n: int) -> int:
    """
    Computes n % 10
    """
    mod = 0
    quo = -1
    count = 0
    while count < n:
        quo += 1
        count += 10
    mod = n - quo * 10
    if mod == 10:
        return 0
    return mod"#
}

#[test]
fn if_no_else() {
    let pycontext = PyContext {
        main: "mod_10".into(),
        functions: BTreeMap::from([
            ("mod_10".into(), mod_10_str().into()),
        ])
        .into(),
    };
    let code = translate(&pycontext);
    println!("{code}")
}
