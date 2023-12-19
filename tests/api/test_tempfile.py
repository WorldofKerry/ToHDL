import tempfile
from importlib import util

raw = """
from python2verilog import verilogify
ns = {}
@verilogify(namespace=ns)
def fib() -> int:
  a, b = 0, 1
  while a < 30:
    yield a
    a, b = b, a + b
"""

# Create a temporary source code file
with tempfile.NamedTemporaryFile(suffix=".py") as tmp:
    tmp.write(raw.encode())
    tmp.flush()

    # Now load that file as a module
    spec = util.spec_from_file_location("tmp", tmp.name)
    module = util.module_from_spec(spec)
    spec.loader.exec_module(module)

    # ...or, while the tmp file exists, you can query it externally
    import inspect

    print(inspect.getsource(module.fib))
    print(module.ns)
