import unittest

from python2verilog.ir import Context, Var
from python2verilog.utils.generics import GenericRepr, pretty_dict


class TestGenericReprAndStr(unittest.TestCase):
    def test_pretty_dict(self):
        dic = {
            "before": "value",
            "outer": {
                "before": "value",
                "middle": {
                    "before": "value",
                    "inner": {"before": "value", "after": "value"},
                    "after": "value",
                },
                "after": "value",
            },
            "after": "value",
        }
        stringed = pretty_dict(dic)
        # logging.error("\n" + stringed)

        self.assertIn("before", stringed)
        self.assertIn("after", stringed)

        self.assertIn("outer", stringed)
        self.assertIn("inner", stringed)
        self.assertIn("middle", stringed)

        self.assertIn("value", stringed)

    def test_generic_repr(self):
        return
        context = Context("bruv")

        # StringIO do not have reconstructable __repr__
        context.module_file = None
        context.testbench_file = None
        copy = eval(repr(context))

        self.assertEqual(context, copy)
