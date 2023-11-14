def func0(n00):
    i00 = 0
    i10 = i00
    if (i10 < n00):
        if (i10 % 2):
            yield i10
            yield from func1(i10, n00)
        else:
            i20 = (i10 + 1)
            i11 = i20
            if (i11 < n00):
                if (i11 % 2):
                    yield i11
                    yield from func1(i11, n00)
                else:
                    yield from func3(i11, n00)
            else:
                return 0
    else:
        return 0

def func1(i10, n00):
    i20 = (i10 + 1)
    i11 = i20
    if (i11 < n00):
        if (i11 % 2):
            yield i11
            yield from func1(i11, n00)
        else:
            i21 = (i11 + 1)
            yield from func2(i20, n00)
    else:
        return 0

def func2(i10, n00):
    if (i10 < n00):
        if (i10 % 2):
            yield i10
            yield from func1(i10, n00)
        else:
            i20 = (i10 + 1)
            i11 = i20
            if (i11 < n00):
                if (i11 % 2):
                    yield i11
                    yield from func1(i11, n00)
                else:
                    yield from func3(i11, n00)
            else:
                return 0
    else:
        return 0

def func3(i10, n00):
    i20 = (i10 + 1)
    i11 = i20
    if (i11 < n00):
        if (i11 % 2):
            yield i11
            yield from func1(i11, n00)
        else:
            i21 = (i11 + 1)
            yield from func2(i20, n00)
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
