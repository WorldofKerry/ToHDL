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


def func1(b10, a00, a000, b100):
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


def main():
    inputs = tuple()
    gen = func0(*inputs)
    for val in itertools.islice(gen, 500):
        print(val, end=", ")


if __name__ == "__main__":
    main()
