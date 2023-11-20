def func0():
    a00 = 0
    if a00 > 1:
        b00 = 10
        b11 = b00
        yield a00
        yield from func1(b11, a00)
    else:
        b40 = 11
        yield b40
        b10 = b40
        yield from func5(a00, b10)


def func1(b10, a00):
    yield b10
    if b10 % 10:
        yield from func4(a00, b10)
    else:
        b30 = a00 + 2
        a20 = a00
        b20 = b30
        yield from func2(a20, b20)


def func2(a20, b20):
    yield a20
    yield from func3(b20)


def func3(b20):
    yield b20


def func4(a00, b10):
    yield a00
    a10 = 15
    a20 = a10
    b20 = b10
    yield from func2(a20, b20)


def func5(a00, b10):
    yield a00
    yield from func1(b10, a00)


import itertools


def get_expected(args, num_iters):
    def even_fib():
        a = 0
        if a > 1:
            b = 10
        else:
            b = 11
            yield b
        yield a
        yield b
        if b % 10:
            yield a
            a = 15
        else:
            b = a + 2
        yield a
        yield b

    inst = even_fib(*args)
    return itertools.islice(inst, num_iters)


def main():
    inputs = tuple()
    gen = func0(*inputs)

    num_iters = 500
    print("Actual: ", end="")
    for val in itertools.islice(gen, num_iters):
        print(val, end=", ")
    print("\nExpected: ", end="")
    for val in get_expected(inputs, num_iters):
        print(val, end=", ")


if __name__ == "__main__":
    main()
