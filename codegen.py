def func0(temp01, n01, temp00, n00, i10, a10, b10, temp10):
    i00 = 0
    a00 = 0
    b00 = 1
    i11 = i00
    a11 = a00
    b11 = b00
    temp11 = temp01
    if a11 < n01:
        if a11 % 2:
            yield a11
            temp20 = a11 + b11
            a20 = b11
            b20 = temp20
            i20 = i11 + 1
            yield from func1(i20, a20, b20, temp20, i11, a11, b11, temp11, n01)
        else:
            temp21 = a11 + b11
            a21 = b11
            b21 = temp21
            i21 = i11 + 1
            yield from func1(i21, a21, b21, temp21, i11, a11, b11, temp11, n01)
    else:
        return 0


def func1(i11, a11, b11, temp11, i10, a10, b10, temp10, n00):
    if a11 < n00:
        if a11 % 2:
            yield a11
            temp20 = a11 + b11
            a20 = b11
            b20 = temp20
            i20 = i11 + 1
            yield from func1(i20, a20, b20, temp20, i11, a11, b11, temp11, n00)
        else:
            temp21 = a11 + b11
            a21 = b11
            b21 = temp21
            i21 = i11 + 1
            yield from func1(i21, a21, b21, temp21, i11, a11, b11, temp11, n00)
    else:
        return 0


import itertools


def main():
    inputs = (123456, 100, 0, 0, 0, 0, 0, 0)
    gen = func0(*inputs)
    for val in itertools.islice(gen, 500):
        print(val, end=", ")


if __name__ == "__main__":
    main()
