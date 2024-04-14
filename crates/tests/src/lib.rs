use tohdl_ir::graph::CFG;

fn code_to_graph(code: &str) -> CFG {
    tohdl_frontend::AstVisitor::from_text(code).get_graph()
}

pub fn aug_assign_str() -> &'static str {
    r#"
def aug_assign(a, b):
    a += 5
    return a
"#
}

pub fn aug_assign_graph() -> CFG {
    code_to_graph(aug_assign_str())
}

pub fn func_call_str() -> &'static str {
    r#"
def func_call(a):
    c = 3
    b = aug_assign(a, c)
    d = return_literal()
    return b + d
"#
}

pub fn func_call_graph() -> CFG {
    code_to_graph(func_call_str())
}

pub fn return_literal_str() -> &'static str {
    r#"
def return_literal():
    return 3
"#
}

pub fn while_loop_str() -> &'static str {
    r#"
def while_loop(n):
    i = 0
    while i < n:
        i += 1
        j = 0
        while j < 10:
            count = 0
            while count < n:
                count += 10
            mod = n - quo * 10
            if mod == 10:
                temp = 0
            else:
                temp = mod
            j += 1
    return 0
"#
}

pub fn while_loop_graph() -> CFG {
    code_to_graph(while_loop_str())
}

pub fn return_literal_graph() -> CFG {
    code_to_graph(return_literal_str())
}

pub fn seven_seg_str() -> &'static str {
    r#"
def seven_seg(n: int) -> int:
    """
    Decimal digit to 7 segment display
    0 means on, 1 means off
    """
    if n == 0:
        return 192
    else:
        return 0
"#
}

pub fn mod_10_str() -> &'static str {
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
    else:
        return mod
    "#
}

pub fn div_10_str() -> &'static str {
    r#"
def div_10(n: int) -> int:
    """
    Computes floor(n / 10)
    """
    mod = 0
    quo = -1
    count = 0
    while count < n:
        quo += 1
        count += 10
    mod = n - quo * 10
    if mod == 10:
        return quo + 1
    return quo
    "#
}

pub fn binary_to_7_seg_str() -> &'static str {
    r#"
def binary_to_7_seg(n: int) -> int:
    """
    Converts binary number to 7-segment display
    """
    ret = 0

    count = 0
    while count < 4:

        mod = mod_10(n)
        n = div_10(n)

        hex0 = seven_seg(mod)

        temp = (hex0 << (count * 7)) & 127
        ret = ret | temp

        count += 1

    return ret
    "#
}

pub fn fib_to_7_seg_str() -> &'static str {
    r#"
def fib_to_7_seg(n):
    a = 0
    b = 1

    count = 0
    while count < n:
        h = binary_to_7_seg(a)
        yield h

        temp = b
        b = a + b
        a = temp
        # a, b = b, a + b

        count += 1
    return 0
    "#
}

pub fn fib_to_7_seg_graph() -> CFG {
    code_to_graph(fib_to_7_seg_str())
}

pub fn caller_str() -> &'static str {
    r#"
def caller(a: int, n: int):
    count = 0
    while count < n:
        c = callee(a, count)
        yield c
        count += 1
    "#
}

pub fn callee_str() -> &'static str {
    r#"
def callee(a: int, b: int) -> int:
    c = a + b
    return 5
    "#
}
