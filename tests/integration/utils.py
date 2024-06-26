"""
Utilities for integration tests
"""

import dataclasses
import itertools
from types import FunctionType
from typing import Iterator, Union, cast

from parameterized import param, parameterized


def make_tuple(input: Union[int, tuple[int, ...]]) -> tuple[int, ...]:
    """
    Makes input into a tuple if it is not a tuple
    """
    if not isinstance(input, tuple):
        return (input,)
    return input


def name_func(testcase_func: FunctionType, _param_num: int, param: param) -> str:
    """
    Custom name function for test case naming with the parameterized package

    Stores in _testMethodName
    """
    test_param = cast(Parameters, param.args[0])
    return parameterized.to_safe_name(
        f"{testcase_func.__name__}::{test_param.func.__name__}"
    )


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
        default_factory=lambda: Parameters.zero_and_exp2s(4)
    )

    # This results in `len(args_list) * len(opti_levels)` test cases

    @staticmethod
    def _exp2s(high: int) -> Iterator[int]:
        """
        Iterates over the exp2, until result is >= high
        """
        for i in itertools.count():
            val = 2**i
            yield val
            if val >= high:
                return

    @staticmethod
    def zero_and_exp2s(high: int) -> list[int]:
        """
        Returns a list of [0, 1, 2, 4, ..., high], where each element except first is a power of 2
        """
        return [0] + list(Parameters._exp2s(high))
