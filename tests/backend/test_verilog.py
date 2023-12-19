import ast
import unittest

from python2verilog import ir
from python2verilog.backend.verilog import CodeGen
from python2verilog.backend.verilog.ast import Instantiation, Module, PosedgeSyncAlways


def assert_lines(test_case: unittest.TestCase, first: str, second: str):
    """
    Asserts that each stripped line in first matches each stripped line in second
    """
    test_case.assertTrue(isinstance(first, str) and isinstance(second, str))
    first_lines, second_lines = first.splitlines(), second.splitlines()
    assert len(first_lines) == len(second_lines)
    for a_line, b_line in zip(first_lines, second_lines):
        test_case.assertEqual(a_line.strip(), b_line.strip())


class TestHelpers(unittest.TestCase):
    def test_verilog_helpers(self):
        code = "def func(a, b, c, d) -> tuple[int, int]:\n  yield(a, b)\n  yield(c, d)"
        ast.parse(code)
        # module = Verilog.create_module_from_python(tree.body[0])
        # warnings.warn(module.to_lines()[0].to_string())


class TestVerilog(unittest.TestCase):
    def test_module(self):
        pass
        # module = Module("cool_name", ["in0"], ["out0"])
        # module.to_lines()
        # TODO: redesign
        # assert_lines(
        #     self,
        #     lines.to_string(),
        #     "module cool_name ( \n input wire signed [31:0] in0,\n input wire _start, \n input wire _clock,\n input wire _reset, \n input wire _wait, \n output reg signed [31:0] out0, \n output reg _ready, \n output reg _valid  \n ); \n endmodule",
        # )

    def test_always(self):
        always = PosedgeSyncAlways(ir.Expression("_clock"))
        always.to_lines()
        # assert_lines(
        #     self,
        #     lines.to_string(),
        #     "always @(posedge _clock) begin \n _valid <= 0; \n end",
        # )

    def test_constructor(self):
        code = """
def circle_lines(s_x, s_y, height) -> tuple[int, int]:
    x = 0
    y = height
    d = 3 - 2 * height
    yield (s_x + x, s_y + y)
    yield (s_x + x, s_y - y)
    yield (s_x - x, s_y + y)
    yield (s_x - x, s_y - y)
    yield (s_x + y, s_y + x)
    yield (s_x + y, s_y - x)
    yield (s_x - y, s_y + x)
    yield (s_x - y, s_y - x)
    while y >= x:
        x = x + 1
        if d > 0:
            y = y - 1
            d = d + 4 * (x - y) + 10
        else:
            d = d + 4 * x + 6
        # yield (x, y, d)
        yield (s_x + x, s_y + y)
        yield (s_x + x, s_y - y)
        yield (s_x - x, s_y + y)
        yield (s_x - x, s_y - y)
        yield (s_x + y, s_y + x)
        yield (s_x + y, s_y - x)
        yield (s_x - y, s_y + x)
        yield (s_x - y, s_y - x)
"""
        tree = ast.parse(code)
        tree.body[0]
        # ir = Generator2List(function)
        # verilog = CodeGen()
        # verilog.from_list_ir(ir.get_root(), ir.get_context())
        # warnings.warn(verilog.get_module())
        # warnings.warn(
        #     verilog.get_testbench_improved([(17, 23, 15), (4, 5, 6), (1, 2, 3)])
        #     .to_lines()
        #     .to_string()
        # )

    def test_instantiation(self):
        ports = {"_clock": "CLK", "_valid": "_valid"}
        Instantiation("module0", "my_module", ports)
        # warnings.warn(inst.to_lines())


import networkx as nx
from matplotlib import pyplot as plt

from python2verilog import ir


class TestNewGraphIR(unittest.TestCase):
    def test_basics(self):
        python = """
def fib(n: int) -> tuple[int]:
    a = 0
    b = 1
    c = 0
    count = 1
    while count < n:
        count += 1
        a = b
        b = c
        c = a + b
        yield (c,)
"""
        # func = ast.parse(python).body[0]
        # inst = Generator2Graph(ir.Context(test_cases=[10]), func)

        # verilog = CodeGen(inst.root, inst.context)
        # module = verilog.get_module_lines()
        # self.assertNotEqual(module, "")
        # testbench = verilog.new_testbench([(10,)]).to_lines().to_string()
        # self.assertNotEqual(testbench, "")
