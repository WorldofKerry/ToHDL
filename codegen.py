def func0(temp00, n00):
    i00 = 0
    a00 = 0
    b00 = 1
    i10 = i00
    a10 = a00
    b10 = b00
    temp10 = temp00
    if a10 < n00:
        if a10 % 2:
            yield a10
            yield from func2(a10, b10, i10, n00)
        else:
            temp20 = a10 + b10
            a20 = b10
            b20 = temp20
            i20 = i10 + 1
            yield from func1(i20, a20, b20, temp20, n00)
    else:
        return 0


def func1(i10, a10, b10, temp10, n00):
    if a10 < n00:
        if a10 % 2:
            yield a10
            yield from func2(a10, b10, i10, n00)
        else:
            temp20 = a10 + b10
            a20 = b10
            b20 = temp20
            i20 = i10 + 1
            yield from func1(i20, a20, b20, temp20, n00)
    else:
        return 0


def func2(a10, b10, i10, n00):
    temp20 = a10 + b10
    a20 = b10
    b20 = temp20
    i20 = i10 + 1
    yield from func1(i20, a20, b20, temp20, n00)


import itertools


def main():
    inputs = (
        123456,
        100,
    )
    gen = func0(*inputs)
    for val in itertools.islice(gen, 500):
        print(val, end=", ")


if __name__ == "__main__":
    main()
