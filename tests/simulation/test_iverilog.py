import unittest

from python2verilog.simulation.iverilog import make_cmd


class TestIVerilog(unittest.TestCase):
    def test_basics(self):
        result = make_cmd("name", ["a.sv", "b.sv"])
        self.assertTrue(result)
