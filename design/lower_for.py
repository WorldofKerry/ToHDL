def odd_fib(n):
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


def verbose():
    sum = 0
    inst = odd_fib(50)
    try:
        val = inst.__next__()
        done = False
    except:
        done = True
    while not done:
        sum += val
        try:
            val = inst.__next__()
            done = False
        except:
            done = True
    return sum


def concise():
    sum = 0
    for val in odd_fib(50):
        sum += val
    return sum


print(verbose())
print(concise())
