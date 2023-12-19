import logging
import subprocess
import unittest

from python2verilog.api.python import py_to_verilog


class TestConvert(unittest.TestCase):
    def test_from_text(self):
        code = """
def fib() -> int:
  a, b = 0, 1
  while a < 30:
    yield a
    a, b = b, a + b
"""
        module, tb = py_to_verilog(
            code, "fib", file_path="./from_text", extra_test_cases=[]
        )
        logging.debug(module)

    def test_help(self):
        CMD = "python3 -m python2verilog --help"
        result = subprocess.run(
            CMD,
            shell=True,
            text=True,
            capture_output=True,
        )
        if result.returncode:
            logging.error(result.stdout + result.stderr)
            raise subprocess.CalledProcessError(CMD, "non-zero return code")

    # def test_module(self):
    #     CMD = "python3 -m python2verilog tests/integration/data/happy_face/python.py \
    #             -o tests/api/module.sv -O5 --name happy_face"
    #     result = subprocess.run(
    #         CMD,
    #         shell=True,
    #         text=True,
    #         capture_output=True,
    #     )
    #     if result.returncode:
    #         logging.error(result.stdout + result.stderr)
    #         raise subprocess.CalledProcessError(CMD, "non-zero return code")

    def test_testbench(self):
        CMD = 'python3 -m python2verilog tests/api/sample.py -n hrange \
                -o tests/api/module.sv -t tests/api/testbench.sv -c "[(0, 1, 10), (3, 7, 25)]"'
        result = subprocess.run(
            CMD,
            shell=True,
            text=True,
            capture_output=True,
        )
        if result.returncode:
            logging.error(result.stdout + result.stderr)
            raise subprocess.CalledProcessError(CMD, "non-zero return code")

    def test_err(self):
        CMD = "python3 -m python2verilog tests/api/sample.py -n hrange \
                -o tests/api/module.sv -t tests/api/testbench.sv"

        def tb_path_no_case():
            result = subprocess.run(
                CMD,
                shell=True,
                text=True,
                capture_output=True,
            )
            if result.returncode:
                raise subprocess.CalledProcessError(
                    cmd=CMD, returncode=result.returncode, stderr="non-zero return code"
                )

        self.assertRaises(subprocess.CalledProcessError, tb_path_no_case)
