import itertools

def func1(i10, n00):
    i20 = (i10 + 1)
    i11 = i20
    if (i11 < n00):
        yield i11
        yield from func1(i11, n00)
    else:
        return 0

def func0(n00):
    i00 = 0
    i10 = i00
    if (i10 < n00):
        yield i10
        yield from func1(i10, n00)
    else:
        return 0

def main():
    inputs = (10,)
    gen = func0(*inputs)
    for val in itertools.islice(gen, 10):
        print(val)

if __name__ == "__main__":
    main()
