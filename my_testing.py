from python2verilog import verilogify


# @verilogify
def foo(a):
    return a


@verilogify
def happy_face(a, b, c):
    c = foo(a)
    d = a + b + c
    yield d


happy_face(1, 2, 3)
