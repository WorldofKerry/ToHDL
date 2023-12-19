import itertools
import unittest

from python2verilog import ir
from python2verilog.backend.verilog import CodeGen
from python2verilog.backend.verilog.codegen import FsmBuilder
from python2verilog.optimizer import IncreaseWorkPerClockCycle, backwards_replace


class TestGraphApplyMapping(unittest.TestCase):
    def test_mapping(self):
        a = ir.Var("a")
        b = ir.Var("b")
        c = ir.Var("c")
        zero = ir.Int(0)
        one = ir.Int(1)

        changes = [a, zero], [b, one], [c, zero], [a, b], [b, c], [c, ir.Add(a, b)]
        """
        a = 0
        b = 1
        c = 0
        a = b
        b = c
        c = a + b
        """

        mapping = {}
        for lvalue, rvalue in changes:
            new_rvalue = backwards_replace(rvalue, mapping)
            mapping[lvalue] = new_rvalue

        self.assertEqual(mapping, {a: one, b: zero, c: ir.Add(one, zero)})
        """
        a = 1
        b = 0
        c = 1 + 0
        """

    def test_combine_var(self):
        a = ir.Var("a")
        b = ir.Var("b")
        c = ir.Var("c")

        count_inst = itertools.count()
        ui = lambda: str(next(count_inst))

        head = ir.ClockedEdge(ui())
        node = head
        node.child = ir.AssignNode(ui(), lvalue=a, rvalue=b)
        node = node.child
        node.child = ir.ClockedEdge(ui())
        node = node.child
        node.child = ir.AssignNode(ui(), lvalue=a, rvalue=c)

        IncreaseWorkPerClockCycle(head.child)
        case = FsmBuilder(head.child, ir.Context()).get_case()
        # logging.error(case.to_string())
        self.assertEqual(len(case.case_items), 1)

    def test_combine_exclusive(self):
        a = ir.ExclusiveVar("a")
        b = ir.ExclusiveVar("b")
        c = ir.ExclusiveVar("c")

        count_inst = itertools.count()
        ui = lambda: str(next(count_inst))

        head = ir.ClockedEdge(ui())
        node = head
        node.child = ir.AssignNode(ui(), lvalue=a, rvalue=b)
        node = node.child
        node.child = ir.ClockedEdge(ui())
        node = node.child
        node.child = ir.AssignNode(ui(), lvalue=a, rvalue=c)

        IncreaseWorkPerClockCycle(head.child)
        case = FsmBuilder(head.child, ir.Context()).get_case()
        # logging.error(case.to_string())
        self.assertEqual(len(case.case_items), 2)

    def test_seq_nonclocked(self):
        """
        a, b = a + 1, a
        should be optimized to
        a = a + 1 => b = a
        NOT
        a = a + a => b = a + 1
        """
        a = ir.Var("a")
        b = ir.Var("b")

        count_inst = itertools.count()
        ui = lambda: str(next(count_inst))

        head = ir.Edge(ui())
        node = head
        node.child = ir.AssignNode(ui(), lvalue=a, rvalue=ir.Add(a, ir.UInt(1)))
        node = node.child
        node.child = ir.NonClockedEdge(ui())
        node = node.child
        node.child = ir.AssignNode(ui(), lvalue=b, rvalue=a)

        IncreaseWorkPerClockCycle(head.child)
        case = FsmBuilder(head.child, ir.Context()).get_case()
        # logging.error("%s", case)
        self.assertTrue("_b <= _a" in str(case))
