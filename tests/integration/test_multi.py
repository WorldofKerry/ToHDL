"""
Test suite for functions that call other functions
"""

import pytest
from parameterized import parameterized

from .base_tests import BaseTestWrapper
from .functions import *
from .utils import Parameters, name_func

PARAMETERS = [
    Parameters(
        func=multi_funcs,
        helpers=[multiplier, p2vrange],
        args_list=[(13, 17)],
    ),
    Parameters(
        func=fib_product,
        helpers=[multiplier, fib, p2vrange],
        args_list=[10, 20],
    ),
    Parameters(
        func=fib,
        helpers=[p2vrange],
        args_list=range(10, 31, 10),
    ),
    Parameters(
        func=quad_multiply,
        helpers=[multiplier_generator],
        args_list=[
            (3, 7),
            (31, 43),
        ],
    ),
    Parameters(
        func=double_for,
        helpers=[p2vrange],
        args_list=[5, 10, 15, 20],
    ),
    Parameters(
        func=dupe,
        helpers=[p2vrange],
        args_list=[(0, 10, 1), (3, 73, 7)],
    ),
    Parameters(
        func=olympic_logo_naive,
        helpers=[circle_lines],
        args_list=[(10, 10, 4), (13, 13, 7)],
    ),
    Parameters(
        func=olympic_logo,
        helpers=[olympic_logo_mids, circle_lines],
        args_list=[(10, 10, 4), (13, 13, 7)],
    ),
    Parameters(
        func=fib_to_7_seg,
        helpers=[binary_to_7_seg, mod_10, div_10, seven_seg, p2vrange],
        args_list=[(10)],
    ),
    Parameters(
        func=binary_to_7_seg,
        helpers=[mod_10, div_10, seven_seg, p2vrange],
        args_list=[(1,)],
    ),
    Parameters(
        func=mod_10,
        helpers=[],
        args_list=[(10)],
    ),
    Parameters(
        func=caller,
        helpers=[callee],
        args_list=[(3, 5)],
    ),
]


@pytest.mark.usefixtures("argparse")
class TestMulti(BaseTestWrapper.BaseTest):
    @parameterized.expand(
        input=PARAMETERS,
        name_func=name_func,
    )
    def test_perf(self, test_param: Parameters):
        BaseTestWrapper.BaseTest.test_perf(self, test_param)

    @parameterized.expand(
        input=PARAMETERS,
        name_func=name_func,
    )
    def test_correct(self, test_param: Parameters):
        BaseTestWrapper.BaseTest.test_correct(self, test_param)

    @classmethod
    def tearDownClass(cls):
        BaseTestWrapper.BaseTest.make_statistics(cls)
