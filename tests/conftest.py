import subprocess
import os
import warnings
import pytest
from .utils import Argument

"""
CLI args for pytest
Use dashes instead of underscores in CLI
"""
params = [
    Argument(
        "first_test",
        default=False,
        action="store_true",
        help="Set to only run first case in each test",
    ),
    Argument(
        "write",
        default=False,
        action="store_true",
        help="Set to write test output files to disk (instead of only in memory)",
    ),
    Argument(
        "synthesis",
        default=False,
        action="store_true",
        help="Set to query synthesis stats",
    ),
    Argument(
        "L",
        "optimization_levels",
        default=[0, 1, 2],
        nargs="+",
        type=int,
        help="Set which optimization levels tests run on",
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


@pytest.fixture()
def argparse(request):
    """
    request is a SubRequest for TestCaseFunction
    """
    args = {}
    for param in params:
        args[param.name] = request.config.getoption(f"--{param.dashed_name}")
    setattr(request.cls, "args", type("Args", (object,), args))
