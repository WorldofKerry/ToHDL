"""
Test suite for functions that call other functions
"""

from types import FunctionType
from typing import Iterable, Union

import pytest
from parameterized import parameterized

from .base_tests import BaseTestWrapper
from .functions import *
from .utils import name_func

PARAMETERS = [
    ([multi_funcs, multiplier, p2vrange], [(13, 17)]),
    ([fib_product, multiplier, fib, p2vrange], [10, 20]),
    ([fib_product, multiplier, fib, p2vrange], [10, 20]),
    ([fib, p2vrange], range(10, 31, 10)),
    ([quad_multiply, multiplier_generator], [(3, 7), (31, 43)]),
    ([double_for, p2vrange], [5, 10, 15, 20]),
    ([dupe, p2vrange], [(0, 10, 1), (3, 73, 7)]),
    ([olympic_logo_naive, circle_lines], [(10, 10, 4), (13, 13, 7)]),
    (
        [olympic_logo, olympic_logo_mids, circle_lines],
        [(10, 10, 4), (13, 13, 7)],
    ),
]


@pytest.mark.usefixtures("argparse")
class TestMulti(BaseTestWrapper.BaseTest):
    @parameterized.expand(
        input=PARAMETERS,
        name_func=name_func,
    )
    def test_perf(
        self,
        funcs: Iterable[FunctionType],
        test_cases: Iterable[Union[tuple[int, ...], int]],
    ):
        BaseTestWrapper.BaseTest.test_perf(self, funcs, test_cases)

    @parameterized.expand(
        input=PARAMETERS,
        name_func=name_func,
    )
    def test_correct(
        self,
        funcs: Iterable[FunctionType],
        test_cases: Iterable[Union[tuple[int, ...], int]],
    ):
        BaseTestWrapper.BaseTest.test_correct(self, funcs, test_cases)

    @classmethod
    def tearDownClass(cls):
        BaseTestWrapper.BaseTest.make_statistics(cls)
