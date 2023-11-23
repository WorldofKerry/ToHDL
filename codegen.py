def func0(n20):
    i00 = 0
    a00 = 0
    b00 = 1
    a20 = a00
    b30 = b00
    i30 = i00
    if (a20 < n20):
        if (a20 % 2):
            yield a20
            temp00 = (a20 + b30)
            a10 = b30
            b10 = temp00
            i10 = (i30 + 1)
            yield from func1(a10, b10, i10, n20)
        else:
            temp01 = (a20 + b30)
            a11 = b30
            b11 = temp01
            i11 = (i30 + 1)
            yield from func1(a11, b11, i11, n20)
    else:
        yield 123

def func1(a20, b30, i30, n20):
    if (a20 < n20):
        if (a20 % 2):
            yield a20
            temp00 = (a20 + b30)
            a10 = b30
            b10 = temp00
            i10 = (i30 + 1)
            yield from func1(a10, b10, i10, n20)
        else:
            temp01 = (a20 + b30)
            a11 = b30
            b11 = temp01
            i11 = (i30 + 1)
            yield from func1(a11, b11, i11, n20)
    else:
        yield 123


import itertools

def get_expected(args, num_iters):
    def even_fib(n):
        i = 0
        a = 0
        b = 1
        while a < n:
            if a % 2:
                yield a
            temp = a + b
            a = b
            b = temp
            i = i + 1
        yield 123

    inst = even_fib(*args)
    return list(itertools.islice(inst, num_iters))

def main():
    inputs = (100,)
    gen = func0(*inputs)

    num_iters = 500
    actual = list(itertools.islice(gen, num_iters))
    print(f"{actual} <- actual")
    expected = get_expected(inputs, num_iters)
    print(f"{expected} <- expected")
    assert(actual == expected)


if __name__ == "__main__":
    main()
