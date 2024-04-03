"""
Base classes for integration tests
"""

import logging
import os
import re
import subprocess
import unittest
from pathlib import Path
from types import FunctionType
from typing import Iterable, Union

import pandas as pd

from python2verilog import (
    Modes,
    get_actual_raw,
    get_context,
    get_expected,
    namespace_to_file,
    namespace_to_verilog,
    verilogify,
)
from python2verilog.backend.verilog.config import CodegenConfig
from python2verilog.simulation import iverilog
from python2verilog.simulation.display import strip_ready, strip_valid

from .utils import make_tuple


class BaseTestWrapper:
    class BaseTest(unittest.TestCase):
        def __init_subclass__(cls) -> None:
            cls.statistics = []
            cls.write = []

        def __test(
            self,
            funcs: Union[Iterable[FunctionType], FunctionType],
            test_cases: Iterable[Union[tuple[int, ...], int]],
            config: CodegenConfig,
        ):
            if not isinstance(funcs, list):
                funcs = [funcs]

            if self.args.first_test:
                test_cases = [next(iter(test_cases))]
            test_cases = list(map(make_tuple, test_cases))

            for opti_level in self.args.optimization_levels:
                ns = {}

                for func in reversed(funcs):  # First function is tested upon
                    verilogified = verilogify(
                        namespace=ns,
                        optimization_level=opti_level,
                        mode=Modes.OVERWRITE,
                    )(func)

                for case in test_cases:
                    verilogified(*case)

                module, testbench = namespace_to_verilog(ns, config)

                logging.debug("Testing Verilog")

                test_name = str(
                    Path(__file__).parent
                    / (self.__dict__["_testMethodName"] + f"::O{opti_level}")
                )
                if self.args.write:
                    BaseTestWrapper.write = True
                    file_stem = test_name.replace("::", "_")
                    namespace_to_file(file_stem, ns, config)
                    context = get_context(verilogified)
                    cmd = iverilog.make_cmd(
                        context.testbench_name,
                        [file_stem + ".sv", file_stem + "_tb.sv"],
                    )
                    logging.info(cmd)

                logging.debug("Getting expected")
                expected = list(get_expected(verilogified))

                logging.debug("Getting actual")
                actual_with_invalid = list(
                    strip_ready(
                        get_actual_raw(
                            verilogified,
                            module,
                            testbench,
                            timeout=1 + len(expected) // 8000,
                        )
                    )
                )
                actual = list(strip_valid(actual_with_invalid))
                logging.info(
                    f"Test cases {test_cases}, actual output length of {len(actual)}"
                )
                logging.info(
                    f"{str(actual[:min(len(actual), 30)]).replace(' ', '')[:-1]},...]"
                )
                logging.info(
                    str(actual_with_invalid[: min(len(actual_with_invalid), 15)])
                    .replace(" ", "")
                    .replace("'", "")[:-1]
                    + ",...]"
                )

                if self.args.write:
                    with open(file_stem + "_expected.csv", mode="w") as f:
                        f.write(
                            "\n".join(
                                map(
                                    lambda x: str(
                                        int(x)
                                        if not isinstance(x, tuple)
                                        else str(tuple(int(y) for y in x))
                                    ),
                                    expected,
                                )
                            )
                        )
                    with open(file_stem + "_actual.csv", mode="w") as f:
                        f.write("\n".join(map(str, actual)))

                self.assertTrue(len(actual) > 0, f"{actual} {expected}")
                self.assertListEqual(actual, expected)

                statistics = {
                    "Test Name": test_name.split("/")[-1],
                    "Py Yields": len(expected),
                    "Ver Clks": len(actual_with_invalid),
                }
                if self.args.synthesis and self.args.write:
                    cmd = " ".join(
                        [
                            "./extern/yosys/oss-cad-suite/bin/yosys",
                            "-QT",
                            "-fverilog",
                            file_stem + ".sv",
                            "-p",
                            "'proc; opt; fsm; opt; stat'",
                        ]
                    )
                    logging.info(f"Running yosys for synthesis {cmd}")
                    stdout = subprocess.check_output(
                        cmd,
                        shell=True,
                        text=True,
                    )
                    stats = stdout[stdout.find("Printing statistics.") :]

                    def snake_case(text):
                        return re.sub(r"[\W_]+", "_", text).strip("_").lower()

                    lines = stats.strip().splitlines()
                    data = {}

                    for line in lines:
                        if ":" in line:
                            key, value = line.split(":")
                            key = snake_case(key).split("number_of_")[-1]
                            value = int(value.strip())
                        else:
                            try:
                                index = line.find("$") + 10
                                value = int(line[index:].strip())
                                key = line[:index].strip()[1:]
                                data[key] = value

                            except ValueError:
                                continue

                        data[key] = value
                    for key, value in data.items():
                        statistics[key] = value

                self.__class__.statistics.append(statistics)

        def test_perf(
            self,
            funcs: Union[Iterable[FunctionType], FunctionType],
            test_cases: Iterable[Union[tuple[int, ...], int]],
        ):
            """
            Performance test
            Does max-clock cycle checks
            The caller is always ready to receive data
            """
            self.__test(funcs, test_cases, CodegenConfig(random_ready=False))

        def test_correct(
            self,
            funcs: Union[Iterable[FunctionType], FunctionType],
            test_cases: Iterable[Union[tuple[int, ...], int]],
        ):
            """
            Correctness test
            The caller is not always ready to receive data
            """
            self.__test(funcs, test_cases, CodegenConfig(random_ready=True))

        @staticmethod
        def make_statistics(cls):
            if cls.statistics:
                cls.statistics.sort(key=lambda e: e["Test Name"])
                df = pd.DataFrame(cls.statistics, columns=cls.statistics[0].keys())
                title = f" Statistics for {cls.__name__} "
                table = df.to_markdown()
                table_width = len(table.partition("\n")[0])
                pad = table_width - len(title)
                result = (
                    "\n"
                    + "=" * (pad // 2)
                    + title
                    + "=" * (pad // 2 + pad % 2)
                    + "\n"
                    + table
                )
                logging.warning(result)
                if cls.write:
                    stats_file_name = (
                        os.path.commonprefix(
                            list(map(lambda e: e["Test Name"], cls.statistics))
                        ).replace("::", "")
                        + ".csv"
                    )
                    with open(Path(__file__).parent / stats_file_name, mode="w") as f:
                        f.write(df.to_csv(index=False))
            else:
                logging.error("Statistics are empty")
