def func0(n0):
    i0 = 0
    a0 = 0
    b0 = 1
    i1 = i0
    a1 = a0
    b1 = b0
    temp0 = temp
    if (a1 < n0):
        if (a1 % 2):
            yield a1
            temp1 = (a1 + b1)
            a2 = b1
            b2 = temp1
            i2 = (i1 + 1)
            yield from func1(i2, a2, b2, temp1, n0)
        else:
            temp1 = (a1 + b1)
            a2 = b1
            b2 = temp1
            i2 = (i1 + 1)
            yield from func1(i2, a2, b2, temp1, n0)
    else:
        return 0

def func1(i1, a1, b1, temp0):
    if (a1 < n0):
        if (a1 % 2):
            yield a1
            temp1 = (a1 + b1)
            a2 = b1
            b2 = temp1
            i2 = (i1 + 1)
            yield from func1(i2, a2, b2, temp1, n0)
        else:
            temp1 = (a1 + b1)
            a2 = b1
            b2 = temp1
            i2 = (i1 + 1)
            yield from func1(i2, a2, b2, temp1, n0)
    else:
        return 0


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
