import itertools

def func0(n00, i100):
    i00 = 0
    i10 = i00
    if (i10 < n00):
        if (i10 % 2):
            yield i10
            yield from func3(i10, n00, i100)
        else:
            i20 = (i10 + 1)
            i11 = i20
            if (i11 < n00):
                if (i11 % 2):
                    yield i11
                    yield from func3(i11, n00, i100)
                else:
                    i21 = (i11 + 1)
                    yield from func4(i21, n00, i100)
            else:
                return 0
    else:
        return 0

def func1(i10, n00, i100):
    i20 = (i10 + 1)
    i11 = i20
    if (i11 < n00):
        if (i11 % 2):
            yield i11
            yield from func3(i11, n00, i100)
        else:
            i21 = (i11 + 1)
            yield from func4(i21, n00, i100)
    else:
        return 0

def func2(i10, n00, i100):
    if (i10 < n00):
        if (i10 % 2):
            yield i10
            yield from func3(i10, n00, i100)
        else:
            i20 = (i10 + 1)
            i11 = i20
            if (i11 < n00):
                if (i11 % 2):
                    yield i11
                    yield from func3(i11, n00, i100)
                else:
                    i21 = (i11 + 1)
                    yield from func4(i21, n00, i100)
            else:
                return 0
    else:
        return 0

def func3(i10, n00, i100):
    i20 = (i10 + 1)
    i11 = i20
    if (i11 < n00):
        if (i11 % 2):
            yield i11
            yield from func3(i11, n00, i100)
        else:
            i21 = (i11 + 1)
            yield from func4(i21, n00, i100)
    else:
        return 0

def func4(i10, n00, i100):
    if (i10 < n00):
        if (i10 % 2):
            yield i10
            yield from func3(i10, n00, i100)
        else:
            i20 = (i10 + 1)
            i11 = i20
            if (i11 < n00):
                if (i11 % 2):
                    yield i11
                    yield from func3(i11, n00, i100)
                else:
                    i21 = (i11 + 1)
                    yield from func4(i21, n00, i100)
            else:
                return 0
    else:
        return 0

def main():
    inputs = (20, 0)
    gen = func0(*inputs)
    for val in itertools.islice(gen, 30):
        print(val, end=", ")

if __name__ == "__main__":
    main()
