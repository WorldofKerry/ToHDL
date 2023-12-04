from python2verilog import verilogify

# @verilogify
def fib(n):
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
    # return 0
print(list(fib(10)))
