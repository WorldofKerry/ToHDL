import unittest

from python2verilog.utils.env import is_debug_mode, set_debug_mode


class TestEnvVars(unittest.TestCase):
    def test_env_vars(self):
        previous_mode = is_debug_mode()

        set_debug_mode(True)
        self.assertTrue(is_debug_mode())
        set_debug_mode(False)
        self.assertFalse(is_debug_mode())

        set_debug_mode(previous_mode)
        self.assertEqual(previous_mode, is_debug_mode())
