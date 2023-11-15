def func0(temp00, n00):
    i00 = 0
    a00 = 0
    b00 = 1
    i10 = i00
    a10 = a00
    b10 = b00
    temp10 = temp00
    if (a10 < n00):
        if (a10 % 2):
            yield a10
            yield from func5(a10, b10, i10, n00)
        else:
            temp20 = (a10 + b10)
            a20 = b10
            b20 = temp20
            i20 = (i10 + 1)
            i11 = i20
            a11 = a20
            b11 = b20
            temp11 = temp20
            if (a11 < n00):
                yield from func1(a11, b11, i11, n00)
            else:
                return 0
    else:
        return 0

def func1(a10, b10, i10, n00):
    if (a10 % 2):
        yield a10
        yield from func5(a10, b10, i10, n00)
    else:
        temp20 = (a10 + b10)
        a20 = b10
        b20 = temp20
        i20 = (i10 + 1)
        i11 = i20
        a11 = a20
        b11 = b20
        temp10 = temp20
        if (a11 < n00):
            if (a11 % 2):
                yield a11
                yield from func5(a11, b11, i11, n00)
            else:
                yield from func2(a11, b11, i11, n00)
        else:
            return 0

def func2(a10, b10, i10, n00):
    temp20 = (a10 + b10)
    a20 = b10
    b20 = temp20
    i20 = (i10 + 1)
    i11 = i20
    a11 = a20
    b11 = b20
    temp10 = temp20
    if (a11 < n00):
        if (a11 % 2):
            yield a11
            yield from func5(a11, b11, i11, n00)
        else:
            yield from func3(a11, b11, i11, n00)
    else:
        return 0

def func3(a10, b10, i10, n00):
    temp20 = (a10 + b10)
    a20 = b10
    b20 = temp20
    i20 = (i10 + 1)
    i11 = i20
    a11 = a20
    b11 = b20
    temp10 = temp20
    if (a11 < n00):
        if (a11 % 2):
            yield a11
            yield from func5(a11, b11, i11, n00)
        else:
            temp21 = (a11 + b11)
            a21 = b11
            b21 = temp21
            i21 = (i11 + 1)
            yield from func4(i21, a21, b21, temp21, n00)
    else:
        return 0

def func4(i10, a10, b10, temp10, n00):
    if (a10 < n00):
        if (a10 % 2):
            yield a10
            yield from func5(a10, b10, i10, n00)
        else:
            temp20 = (a10 + b10)
            a20 = b10
            b20 = temp20
            i20 = (i10 + 1)
            i11 = i20
            a11 = a20
            b11 = b20
            temp11 = temp20
            if (a11 < n00):
                yield from func1(a11, b11, i11, n00)
            else:
                return 0
    else:
        return 0

def func5(a10, b10, i10, n00):
    temp20 = (a10 + b10)
    a20 = b10
    b20 = temp20
    i20 = (i10 + 1)
    i11 = i20
    a11 = a20
    b11 = b20
    temp10 = temp20
    if (a11 < n00):
        if (a11 % 2):
            yield a11
            yield from func5(a11, b11, i11, n00)
        else:
            yield from func3(a11, b11, i11, n00)
    else:
        return 0

import itertools
def main():
    inputs = (123456, 20,)
    gen = func0(*inputs)
    for val in itertools.islice(gen, 10):
        print(val, end=", ")

if __name__ == "__main__":
    main()
