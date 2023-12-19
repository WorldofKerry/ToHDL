import sys

import pytest

from python2verilog.utils import env

from .utils import Argument

"""
CLI args for pytest
Use dashes instead of underscores in CLI
"""
params = [
    Argument(
        "F",
        "first_test",
        default=False,
        action="store_true",
        help="Set to only run first case in each test",
    ),
    Argument(
        "R",
        "write",
        default=False,
        action="store_true",
        help="Set to write test output files to disk (instead of only in memory)",
    ),
    Argument(
        "S",
        "synthesis",
        default=False,
        action="store_true",
        help="Set to query synthesis stats",
    ),
    Argument(
        "L",
        "optimization_levels",
        default=[],
        nargs="+",
        type=int,
        action="extend",
        help="Set which optimization levels tests run on",
    ),
    Argument(
        "I",
        "iverilog_path",
        default="iverilog",
        type=str,
        action="store",
        help="Path to iverilog",
    ),
    Argument(
        "E",
        "env_debug",
        default=False,
        action="store_true",
        help="Set env variables for debugging",
    ),
]
"""
Other useful flags
-s for output print statements (otherwise they are captured)
-v for verbose
-k "filter" to filter tests
--log-cli-level=DEBUG to set logging package
"""


def pytest_addoption(parser: pytest.Parser):
    for param in params:
        param.add_to_parser(parser)


env.set_var(env.Vars.NO_WRITE_TO_FS, "1")


@pytest.fixture()
def argparse(request):
    """
    request is a SubRequest for TestCaseFunction
    """
    args = {}
    for param in params:
        args[param.name] = request.config.getoption(f"--{param.dashed_name}")

    args["optimization_levels"] = (
        set(args["optimization_levels"]) if args["optimization_levels"] else {1}
    )
    sys.setrecursionlimit(
        sys.getrecursionlimit() + max(args["optimization_levels"]) * 250
    )
    env.set_var(env.Vars.IVERILOG_PATH, args["iverilog_path"])
    if args["synthesis"]:
        # Synthesis tool yosys does not support SystemVerilog features
        env.set_var(env.Vars.IS_SYSTEM_VERILOG, None)
    else:
        env.set_var(env.Vars.IS_SYSTEM_VERILOG, "")
    if args["env_debug"]:
        env.set_var(env.Vars.DEBUG_COMMENTS, "1")

    setattr(request.cls, "args", type("Args", (object,), args))
