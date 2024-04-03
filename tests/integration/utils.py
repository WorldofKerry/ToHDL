"""
Utilities for integration tests
"""

import dataclasses
from types import FunctionType
from typing import Union


def make_tuple(input: Union[int, tuple[int, ...]]) -> tuple[int, ...]:
    """
    Makes input into a tuple if it is not a tuple
    """
    if not isinstance(input, tuple):
        return (input,)
    return input


def name_func(testcase_func: FunctionType, _: int, param: dict) -> str:
    """
    Custom name function

    Stores in _testMethodName
    """
    # assert False, f"{param=} {param[0][0]=}"
    test_param = param[0][0]
    return f"{testcase_func.__name__}::{test_param.func.__name__}"


@dataclasses.dataclass
class Parameters:
    # Function to-be-tested
    func: FunctionType

    # List of argument combinations to-be-tested
    # Each element is a separate test case
    # Each element is unpacked as the arguments for that test case
    args_list: list[Union[tuple[int, ...], int]]

    # All functions called by `func`
    helpers: list[FunctionType] = dataclasses.field(default_factory=list)

    # Optimization levels to-be-tested
    opti_levels: list[int] = dataclasses.field(
        default_factory=lambda: [0, 1, 2, 4, 8, 16]
    )

    # This results in `len(args_list) * len(opti_levels)` test cases
