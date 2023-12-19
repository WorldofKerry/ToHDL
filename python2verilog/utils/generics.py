"""
Implementation of generic base classes for __repr__ and __str__
"""


import reprlib
from typing import Any


def pretty_dict(dic: dict[Any, Any], indent: int = 0) -> str:
    """
    Returns pretty-formatted stringified dict
    """
    result = "{\n"
    for key, value in dic.items():
        result += "\t" * (indent + 1) + str(key) + ": "
        if isinstance(value, dict):
            result += pretty_dict(value, indent + 1)
        else:
            result += str(value) + ",\n"
    return result + "\t" * (indent) + "}\n"


class GenericRepr:
    """
    Implements a generic __repr__ based on self.__dict__
    """

    @reprlib.recursive_repr()
    def __repr__(self):
        items = [f"{key}=({repr(value)})" for key, value in self.__dict__.items()]
        return f"{self.__class__.__name__}({','.join(items)})"


class GenericReprAndStr(GenericRepr):
    """
    Implements a generic __repr__ and __str__ based on self.__dict__
    """

    @reprlib.recursive_repr()
    def __str__(self):
        return f"{self.__class__.__name__}\n{pretty_dict(self.__dict__)}"
