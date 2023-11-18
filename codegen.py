def func0():
    a00 = 0
    if (a00 > 1):
        b00 = 10
        b10 = b00
        a10 = 15
    else:
        b20 = 11
        yield b20
        yield from func1(a00, b20)

def func1(a00, b20):
    yield a00
    yield from func2(b20, a00)

def func2(b20, a00, a000, b200):
    yield b20
    if (b20 % 10):
        yield from func5(a00, b20)
    else:
        b30 = (a00 + 2)
        yield from func3(a00, b30)

def func3(a00, b30):
    yield a00
    yield from func4(b30)

def func4(b30):
    yield b30

def func5(a00, b20):
    yield a00
    b10 = b20
    a10 = 15


import itertools


def main():
    inputs = tuple()
    gen = func0(*inputs)
    for val in itertools.islice(gen, 500):
        print(val, end=", ")


if __name__ == "__main__":
    main()
