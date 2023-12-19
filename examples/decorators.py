from python2verilog.api import verilogify


@verilogify
def p2vrange(base: int, limit: int, step: int) -> int:
    """
    Simplified version of Python's built-in range function
    """
    while base < limit:
        yield base
        base += step


@verilogify
def fib(n: int) -> int:
    """
    Fibonacci sequence
    """
    a, b = 0, 1
    for _ in p2vrange(0, n, 1):
        yield a
        a, b = b, a + b


@verilogify
def multiplier(multiplicand: int, multiplier: int) -> int:
    product = 0
    while multiplier > 0:
        product += multiplicand
        multiplier -= 1
    return product


@verilogify
def fib_product(n):
    """
    Yields the product of the first n fibonacci numbers
    """
    for num in fib(n):
        prod = multiplier(num, num)
        yield prod


fib_product(30)
