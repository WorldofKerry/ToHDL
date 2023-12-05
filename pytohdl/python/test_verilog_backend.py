from importlib import util
import tempfile
import traceback
import pytohdl
import inspect


def even_fib(n):
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
    return 0


source_code = inspect.getsource(even_fib)
result = pytohdl.translate(source_code)

print(result)