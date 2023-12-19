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
    if isinstance(param.args[0], FunctionType):
        return f"{testcase_func.__name__}::{param.args[0].__name__}"
    else:
        # If given list of funcs, use first for naming
        return f"{testcase_func.__name__}::{param.args[0][0].__name__}"
