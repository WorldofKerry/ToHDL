"""
Type assertion utilities
"""
from typing import Any, Optional, Type, TypeVar, Union

try:
    from typing import TypeAlias, TypeGuard
except ImportError:
    from typing_extensions import TypeAlias, TypeGuard

_ValueType = TypeVar("_ValueType")  # pylint: disable=invalid-name
_KeyType = TypeVar("_KeyType")  # pylint: disable=invalid-name
_ClassInfo: TypeAlias = Union[type, tuple["_ClassInfo", ...]]


def typed_list(list_: Optional[list[Any]], type_: Type[_ValueType]) -> list[_ValueType]:
    """
    Asserts that all elems in list_ are of type_, then returns list_ or [] if list_ is None
    """
    if list_ is None:
        return []
    assert all(guard(elem, type_) for elem in list_)
    return list_


def guard_dict(
    obj: dict[Any, Any],
    key_type: Type[_KeyType],
    value_type: Type[_ValueType],
) -> TypeGuard[dict[_KeyType, _ValueType]]:
    """
    Asserts that all key, values in dict_ are correctly typed,
    """
    return (
        guard(obj, dict)
        and all(guard(x, key_type) for x in obj.keys())
        and all(guard(x, value_type) for x in obj.values())
    )


def guard(obj: Any, type_: Type[_ValueType]) -> TypeGuard[_ValueType]:
    """
    Type guard for type
    """
    return isinstance(obj, type_)


def typed(
    obj: Union[_ValueType, None], type_: Type[_ValueType]
) -> Union[_ValueType, None]:
    """
    Asserts that obj is of type type_, then returns obj or None if obj is None
    """
    assert obj is None or guard(obj, type_), f"{obj} {type(obj)}"
    return obj


def typed_strict(obj: Union[_ValueType, None], type_: Type[_ValueType]) -> _ValueType:
    """
    Asserts that obj is of type type_
    """
    assert guard(
        obj, type_
    ), f"Expected type {type_} got {type(obj)} instead from {obj}"
    return obj
