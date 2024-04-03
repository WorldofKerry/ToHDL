"""
Test suite for basic functions
"""

import pytest
from parameterized import parameterized

from .base_tests import BaseTestWrapper
from .functions import *
from .utils import Parameters, name_func

PARAMETERS = [
    Parameters(func=keyword_test, args_list=[()]),
    Parameters(func=floor_div, args_list=[13, 23]),
    Parameters(func=operators, args_list=[(31, 13), (-31, 13), (31, -13), (-31, -13)]),
    Parameters(func=multiplier_generator, args_list=[(13, 17), (78, 67), (15, -12)]),
    Parameters(func=multiplier, args_list=[(13, 17), (78, 67), (15, -12)]),
    Parameters(func=p2vrange, args_list=[(0, 10, 1), (0, 1000, 1)]),
    Parameters(func=division, args_list=[(6, 7, 10), (2, 3, 30), (13, 17, 5)]),
    Parameters(func=circle_lines, args_list=[(21, 37, 7), (79, 45, 43)]),
    Parameters(func=happy_face, args_list=[(50, 51, 7), (76, 97, 43)]),
    Parameters(func=rectangle_filled, args_list=[(32, 84, 5, 7), (64, 78, 23, 27)]),
    Parameters(func=rectangle_lines, args_list=[(32, 84, 5, 7), (84, 96, 46, 89)]),
    Parameters(func=floating_point_add, args_list=[(0, 127, 0, 0, 128, 0)]),  # 1 + 2
]


@pytest.mark.usefixtures("argparse")
class TestSingle(BaseTestWrapper.BaseTest):
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
