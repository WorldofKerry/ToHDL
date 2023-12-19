"""
Converts Python generator functions to synthesizable Verilog.

For sample Python inputs,
visit https://github.com/WorldofKerry/Python2Verilog/tree/main/tests/integration/data
"""
import argparse
import ast
import logging
import os

from python2verilog import ir
from python2verilog.api import py_to_verilog

if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawTextHelpFormatter
    )

    parser.add_argument(
        "input_file",
        type=str,
        help="Input file containing a python generator function",
    )

    parser.add_argument(
        "-n",
        "--name",
        type=str,
        help="Name of function to be converted, defaults to python filename stem",
        default="",
        required=True,
    )

    parser.add_argument(
        "-o",
        "--output",
        type=str,
        help="Output file for System Verilog module",
        default="",
    )

    TESTBENCH_ARG = parser.add_argument(
        "-t",
        "--testbench",
        type=str,
        help="Output file for System Verilog testbench",
        default="",
    )

    parser.add_argument(
        "-c",
        "--test-cases",
        type=str,
        help="A list of test cases for testbench. "
        'Required to output testbench. E.g. `-c "[(1, 2, 3, 4)]"`',
        default="",
    )
    parser.add_argument(
        "-O",
        "--optimization-level",
        type=int,
        help="Set to a value greater than 0 for optimizations, "
        "higher values will use more logic cells and do more work per clock cycle",
        default=1,
    )

    verbose_quiet_group = parser.add_mutually_exclusive_group()
    verbose_quiet_group.add_argument("-v", "--verbose", action="count", default=0)
    verbose_quiet_group.add_argument("-q", "--quiet", action="store_const", const=True)

    args = parser.parse_args()

    if not args.quiet:
        if args.verbose >= 2:
            logging.root.setLevel(logging.DEBUG)
        elif args.verbose == 1:
            logging.root.setLevel(logging.INFO)
        else:
            logging.root.setLevel(logging.WARNING)
        logging.basicConfig(format="%(levelname)s %(filename)s:%(lineno)s %(message)s")

    def get_default_tb_filename(stem: str):
        """
        Gets default testbench filename
        """
        return stem + "_tb.sv"

    input_file_path = parser.parse_args().input_file
    input_file_stem = os.path.splitext(input_file_path)[0]
    if args.output == "":
        args.output = input_file_stem + ".sv"

    if args.name == "":
        args.name = input_file_stem
    if args.testbench == "":
        args.testbench = get_default_tb_filename(input_file_stem)

    with open(
        os.path.abspath(input_file_path), mode="r", encoding="utf8"
    ) as python_file:
        python = python_file.read()

        test_cases = ast.literal_eval(args.test_cases) if args.test_cases else []

        logging.info("Extra test cases: %s", test_cases)
        module_str, testbench_str = py_to_verilog(
            code=python,
            function_name=args.name,
            extra_test_cases=test_cases,
            file_path=input_file_path,
            optimization_level=args.optimization_level,
        )

        with open(
            os.path.abspath(args.output), mode="w+", encoding="utf8"
        ) as module_file:
            print(
                "attempting to write",
                module_str[:50],
                "to",
                os.path.abspath(args.output),
            )
            module_file.write(module_str)

        if args.test_cases != "":
            with open(
                os.path.abspath(args.testbench), mode="w+", encoding="utf8"
            ) as tb_file:
                tb_file.write(testbench_str)
        elif args.test_cases == "" and args.testbench != get_default_tb_filename(
            input_file_stem
        ):
            raise argparse.ArgumentError(
                TESTBENCH_ARG,
                f"testbench path provided by no test cases provided \
                    {args.test_cases}, {args.testbench}",
            )
