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


def get_actual(raw):
    with tempfile.NamedTemporaryFile(suffix=".py") as tmp:
        tmp.write(raw.encode())
        tmp.flush()

        spec = util.spec_from_file_location("tmp", tmp.name)
        module = util.module_from_spec(spec)
        spec.loader.exec_module(module)

        print(f"{module=} {dir(module)=}")
        return list(module.func0(0, 100))


actual = get_actual(result)
expected = list(even_fib(100))
assert expected == actual
