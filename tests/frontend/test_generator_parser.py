import ast
import unittest

import networkx as nx
import pytest
from matplotlib import pyplot as plt

from python2verilog import (
    Modes,
    get_actual_raw,
    get_context,
    get_expected,
    ir,
    namespace_to_file,
    namespace_to_verilog,
    verilogify,
)
from python2verilog.backend import verilog
from python2verilog.backend.verilog.codegen import FsmBuilder
from python2verilog.frontend.function import Function
from python2verilog.ir import Context, create_networkx_adjacency_list


class TestGenerator2Graph(unittest.TestCase):
    def blank_generator():
        yield 0

    @pytest.mark.skip(reason="Not implemented yet")
    def test_multi_assign(self):
        ns = {}

        @verilogify(namespace=ns)
        def multi_assign():
            a = 1
            b, c = 2, 3
            yield a, b, c
            # d, e, (x, y), g = 10, 20, (a, b), c

        multi_assign()

        # logging.debug(dis.Bytecode("d, e, (x, y), g = 10, 20, (a, b), c").dis())

        module, testbench = namespace_to_verilog(ns)

    def test_assign_visitor(self):
        code = "d, e, (x, y), g = 10, c, (a, b), 20"

        assign = ast.parse(code).body[0]

        target: ast.Tuple = assign.targets[0]
        value: ast.Tuple = assign.value

        result = Function(ir.Context.empty_valid())._target_value_visitor(target, value)

        self.assertEqual(
            str(list(result)), "[(_d, 10), (_e, _c), (_x, _a), (_y, _b), (_g, 20)]"
        )

        code = "d, e, bruv, g = 10, 20, (a, b), c"

        assign: ast.Assign = ast.parse(code).body[0]

        target: ast.Tuple = assign.targets[0]
        value: ast.Tuple = assign.value

        with self.assertRaises(TypeError):
            result = Function(ir.Context.empty_valid())._target_value_visitor(
                target, value
            )
            list(result)

    def test_basics(self):
        ns = {}

        @verilogify(namespace=ns)
        def my_func():
            # b = 0
            # while b < 20:
            #     if b == 15:
            #         yield b
            #     b = b + 1
            # yield b + 420
            n = 10
            a, b = 0, 1
            count = 1
            while count < n:
                yield a
                a, b = b, a + b
                count += 1

        my_func()

        cxt, root = Function(get_context(my_func)).parse_function()
        case = FsmBuilder(root, cxt).get_case()
        sv = verilog.CodeGen(root, cxt).get_module_str()
        # with open("./new.sv", mode="w") as f:
        #     f.write(str(sv))

        cxt, root = Function(get_context(my_func)).parse_function()
        case = FsmBuilder(root, cxt).get_case()
        sv = verilog.CodeGen(root, cxt).get_module_str()
        # with open("./old.sv", mode="w") as f:
        #     f.write(str(sv))

        # module, testbench = namespace_to_verilog(ns)

        return
