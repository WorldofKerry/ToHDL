def func0(temp00, n00):
    i00 = 0
    a00 = 0
    b00 = 1
    i10 = i00
    a10 = a00
    b10 = b00
    temp10 = temp00
    if (a10 < n00):
        yield a10
        yield from func1(a10, b10, i10, n00)
    else:
        return 0

def func1(a10, b10, i10, n00):
    temp20 = (a10 + b10)
    a20 = b10
    b20 = temp20
    i20 = (i10 + 1)
    i11 = i20
    a11 = a20
    b11 = b20
    temp10 = temp20
    if (a11 < n00):
        yield a11
        yield from func1(a11, b11, i11, n00)
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
