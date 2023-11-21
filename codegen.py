def func0(n00):
    i00 = 0
    a00 = 0
    b00 = 1
    a10 = a00
    b10 = b00
    i10 = i00
    if a10 < n00:
        if a10 % 2:
            yield a10
            temp20 = a10 + b10
            a20 = b10
            b20 = temp20
            i20 = i10 + 1
            yield from func1(a20, b20, i20, n00)
        else:
            temp21 = a10 + b10
            a21 = b10
            b21 = temp21
            i21 = i10 + 1
            yield from func1(a21, b21, i21, n00)
    else:
        yield 0


def func1(a10, b10, i10, n00):
    if a10 < n00:
        if a10 % 2:
            yield a10
            temp20 = a10 + b10
            a20 = b10
            b20 = temp20
            i20 = i10 + 1
            yield from func1(a20, b20, i20, n00)
        else:
            temp21 = a10 + b10
            a21 = b10
            b21 = temp21
            i21 = i10 + 1
            yield from func1(a21, b21, i21, n00)
    else:
        yield 0


import itertools


def main():
    # inputs = tuple()
    inputs = (100,)
    gen = func0(*inputs)
    for val in itertools.islice(gen, 500):
        print(val, end=", ")


if __name__ == "__main__":
    main()
