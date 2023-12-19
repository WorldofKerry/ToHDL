import atexit
import os
import unittest
from functools import wraps
from pathlib import Path

import pytest

from python2verilog.api import Modes, new_namespace, verilogify
from python2verilog.api.context import context_to_verilog, context_to_verilog_and_dump
from python2verilog.api.python import py_to_context


class TestParsePython(unittest.TestCase):
    def test_parse_python(self):
        code = """
def nuts(input, other):
  yield input, input
  yield input + 1, input

def deeznuts(input):
  yield input
  yield input + 1

inst = nuts(50, 51)
inst = nuts(15, 16)
inst = deeznuts(420)
"""
        py_to_context(
            code,
            "nuts",
            Path(__file__).parent / "bruh_nuts",
            write=True,
            optimization_level=1,
        )
        # logging.debug(func_ast)

    # def test_this_file(self):
    #     with open(__file__, mode="r") as f:
    #         # logging.warning(parse_python(f.read(), "test_parse_python"))
    #         self.assertRaises(RuntimeError, parse_python, f.read(), "test_parse_python")

    def test_mix_types(self):
        code = """
def nuts2(input, other):
  yield input, input
  yield input + 1, input

def deeznuts(input):
  yield input
  yield input + 1

inst = nuts2(50, 51)
inst = nuts2(15, "16")
inst = deeznuts(420)
"""
        self.assertRaises(
            AssertionError, py_to_context, code, "nuts2", __file__, True, 1
        )

        code = """
def nuts3(input, other):
  yield input, input
  yield input + 1, "bruv"

def deeznuts(input):
  yield input
  yield input + 1

inst = nuts3(50, 51)
inst = nuts3(15, 16)
inst = deeznuts(420)
"""
        self.assertRaises(
            AssertionError, py_to_context, code, "nuts3", __file__, True, 1
        )


@pytest.mark.usefixtures("argparse")
class TestVerilogify(unittest.TestCase):
    def writes(func):
        """
        Decorator to clean up written .sv files
        """

        @wraps(func)
        def wrapper(self, *args, **kwargs):
            func(self, *args, **kwargs)

            dir = Path(__file__).parent.absolute()
            filenames = os.listdir(dir)
            for filename in filenames:
                if filename.endswith(".sv"):
                    os.remove(os.path.join(dir, filename))

        return wrapper

    @classmethod
    @writes
    def setUp(cls):
        pass

    @writes
    def test_basic(self):
        @verilogify(mode=Modes.WRITE)
        def counter0(n):
            i = 0
            while i < n:
                yield (i)
                i += 1

        @verilogify
        def counter1(n, other):
            i = 0
            while i < n:
                yield (i)
                i += 1

        for value in counter0(5):
            # print(value)
            pass

        counter0(10)
        counter1(15, 20)

    @writes
    def test_overwrite_fail(self):
        @verilogify(mode=Modes.WRITE)
        def counter_overwrite(n):
            i = 0
            while i < n:
                yield (i)
                i += 1

        counter_overwrite(10)

        def inner():
            try:

                @verilogify(
                    mode=Modes.WRITE,
                    namespace=new_namespace(Path(__file__).parent / "other"),
                )
                def counter_overwrite(n):
                    i = 0
                    while i < n:
                        yield (i)
                        i += 1

                counter_overwrite(10)
            except FileExistsError as e:
                raise e

        # self.assertRaises(FileExistsError, inner)

    @writes
    def test_func_call(self):
        ns = new_namespace(Path(__file__).parent / "triple_circle")

        @verilogify(namespace=ns, mode=Modes.OVERWRITE)
        def circle_lines(s_x, s_y, height) -> tuple[int, int, int, int, int, int]:
            x = 0
            y = height
            d = 3 - 2 * y
            yield (s_x + x, s_y + y, height, x, y, d)
            yield (s_x + x, s_y - y, height, x, y, d)
            yield (s_x - x, s_y + y, height, x, y, d)
            yield (s_x - x, s_y - y, height, x, y, d)
            yield (s_x + y, s_y + x, height, x, y, d)
            yield (s_x + y, s_y - x, height, x, y, d)
            yield (s_x - y, s_y + x, height, x, y, d)
            yield (s_x - y, s_y - x, height, x, y, d)
            while y >= x:
                x = x + 1
                if d > 0:
                    y = y - 1
                    d = d + 4 * (x - y) + 10
                else:
                    d = d + 4 * x + 6
                yield (s_x + x, s_y + y, height, x, y, d)
                yield (s_x + x, s_y - y, height, x, y, d)
                yield (s_x - x, s_y + y, height, x, y, d)
                yield (s_x - x, s_y - y, height, x, y, d)
                yield (s_x + y, s_y + x, height, x, y, d)
                yield (s_x + y, s_y - x, height, x, y, d)
                yield (s_x - y, s_y + x, height, x, y, d)
                yield (s_x - y, s_y - x, height, x, y, d)

        @verilogify(namespace=ns, mode=Modes.OVERWRITE, optimization_level=0)
        def triple_circle(centre_x, centre_y, radius):
            # noqa
            c_x = centre_x
            c_y = centre_y
            c_x1 = c_x + radius // 2
            c_y1 = c_y + radius * 2 // 6
            c_x2 = c_x - radius // 2
            c_y2 = c_y + radius * 2 // 6
            c_x3 = c_x
            c_y3 = c_y - radius * 2 // 6

            gen0 = circle_lines(c_x1, c_y1, radius)
            for x, y, a, b, c, d in gen0:
                yield x, y
            # gen1 = circle_lines(c_x2, c_y2, radius)
            # for x, y, a, b, c, d in gen1:
            #     yield x, y
            # gen2 = circle_lines(c_x3, c_y3, radius)
            # for x, y, a, b, c, d in gen2:
            #     yield x, y

        triple_circle(50, 50, 8)
        # warnings.warn(module)
        # warnings.warn(tb)
        module, tb, cytoscape = context_to_verilog_and_dump(ns[triple_circle.__name__])
        with open(Path(__file__).parent / "triple_circle_cytoscape.log", mode="w") as f:
            f.write(str(cytoscape))
        # python3 python2verilog/utils/cytoscape.py  tests/aptriple_circlees_cytoscape.log
        module, tb, cytoscape = context_to_verilog_and_dump(ns[circle_lines.__name__])
        with open(Path(__file__).parent / "circle_lines_cytoscape.log", mode="w") as f:
            f.write(str(cytoscape))
