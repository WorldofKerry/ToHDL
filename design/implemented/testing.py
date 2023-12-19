import logging

logging.root.setLevel(logging.INFO)

from python2verilog.api import convert_file_to_file


def fib(n):
    yield (n)


fib(10)

convert_file_to_file("fib", __file__, overwrite=True)
