import logging
import unittest

from python2verilog.api import Modes, context_to_verilog, get_context, verilogify
from python2verilog.backend.verilog.config import CodegenConfig


class TestVerilogify(unittest.TestCase):
    def test_main(self):
        namespace = {}

        @verilogify(namespace=namespace)
        def count(n):
            i = 0
            while i < n:
                yield (i)
                i += 1

        list(count(10))

        logging.debug("%s", namespace[count.__name__])
        self.assertEqual(count.__name__, get_context(count).name)

        module, testbench = context_to_verilog(
            get_context(count), config=CodegenConfig()
        )
        self.assertIn(count.__name__, testbench)
        self.assertIn(count.__name__, module)
