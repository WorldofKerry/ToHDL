def func0(n00, i100, i1000, n000, i10000):
    i00 = 0
    i10 = i00
    if (i10 < n00):
        if (i1000 % 2):
            yield i1000
            yield from func1(i1000, n000, i10000)
        else:
            i2000 = (i1000 + 1)
            i11 = i2000
            if (i11 < n00):
                if (i1000 % 2):
                    yield i1000
                    yield from func1(i1000, n000, i10000)
                else:
                    yield from func3(i100, n00, i1000)
            else:
                return 0
    else:
        return 0

def func1(i100, n00, i1000, n000, i10000):
    i200 = (i100 + 1)
    i10 = i200
    if (i10 < n00):
        if (i1000 % 2):
            yield i1000
            yield from func1(i1000, n000, i10000)
        else:
            i2010 = (i1000 + 1)
            yield from func2(i2010, n000, i1000, i10000)
    else:
        return 0

def func2(i10, n00, i100, i1000, n000, i10000):
    if (i10 < n00):
        if (i1000 % 2):
            yield i1000
            yield from func1(i1000, n000, i10000)
        else:
            i2000 = (i1000 + 1)
            i11 = i2000
            if (i11 < n00):
                if (i1000 % 2):
                    yield i1000
                    yield from func1(i1000, n000, i10000)
                else:
                    yield from func3(i100, n00, i1000)
            else:
                return 0
    else:
        return 0

def func3(i100, n00, i1000, n000, i10000):
    i200 = (i100 + 1)
    i10 = i200
    if (i10 < n00):
        if (i1000 % 2):
            yield i1000
            yield from func1(i1000, n000, i10000)
        else:
            i2010 = (i1000 + 1)
            yield from func2(i2010, n000, i1000, i10000)
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
