def func0(n00):
    i00 = 0
    a00 = 0
    b00 = 1
    i10 = temp
    a10 = b00
    b10 = a00
    temp00 = i00
    if (a10 < n00):
        if (a10 % 2):
            yield a10
            yield from func1(a10, b10, i10, n00)
        else:
            temp10 = (a10 + b10)
            a20 = b10
            b20 = temp10
            i20 = (i10 + 1)
            i11 = temp10
            a11 = b20
            b11 = a20
            temp01 = i20
            if (a11 < n00):
                if (a11 % 2):
                    yield a11
                    yield from func1(a11, b11, i11, n00)
                else:
                    yield from func3(a11, b11, i11, n00)
            else:
                return 0
    else:
        return 0

def func1(a10, b10, i10, n00):
    temp10 = (a10 + b10)
    a20 = b10
    b20 = temp10
    i20 = (i10 + 1)
    i11 = temp10
    a11 = b20
    b11 = a20
    temp00 = i20
    if (a11 < n00):
        if (a11 % 2):
            yield a11
            yield from func1(a11, b11, i11, n00)
        else:
            temp11 = (a11 + b11)
            a21 = b11
            b21 = temp11
            i21 = (i11 + 1)
            yield from func2(i20, a20, b20, temp10, n00)
    else:
        return 0

def func2(i10, a10, b10, temp00, n00):
    if (a10 < n00):
        if (a10 % 2):
            yield a10
            yield from func1(a10, b10, i10, n00)
        else:
            temp10 = (a10 + b10)
            a20 = b10
            b20 = temp10
            i20 = (i10 + 1)
            i11 = temp10
            a11 = b20
            b11 = a20
            temp01 = i20
            if (a11 < n00):
                if (a11 % 2):
                    yield a11
                    yield from func1(a11, b11, i11, n00)
                else:
                    yield from func3(a11, b11, i11, n00)
            else:
                return 0
    else:
        return 0

def func3(a10, b10, i10, n00):
    temp10 = (a10 + b10)
    a20 = b10
    b20 = temp10
    i20 = (i10 + 1)
    i11 = temp10
    a11 = b20
    b11 = a20
    temp00 = i20
    if (a11 < n00):
        if (a11 % 2):
            yield a11
            yield from func1(a11, b11, i11, n00)
        else:
            temp11 = (a11 + b11)
            a21 = b11
            b21 = temp11
            i21 = (i11 + 1)
            yield from func2(i20, a20, b20, temp10, n00)
    else:
        return 0

import itertools
def main():
    inputs = (20,)
    gen = func0(*inputs)
    for val in itertools.islice(gen, 30):
        print(val, end=", ")

if __name__ == "__main__":
    main()
