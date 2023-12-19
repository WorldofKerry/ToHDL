from __future__ import annotations

from typing import Any, Optional, Sequence

test = (1, (3, 5)), 5, "lmao"
print(test)
print(*test)


class TreeNode:
    """
    Tree Node
    """

    def __init__(self, value: Any, children: Optional[list[TreeNode]] = None) -> None:
        self.value = value
        self.children = [] if children is None else children

    def to_string(self, indent: int = 0) -> str:
        ret = "\t" * indent + f"{self.value}\n"
        for child in self.children:
            ret += child.to_string(indent + 1)
        return ret

    def __str__(self) -> str:
        return self.to_string()

    def __eq__(self, __value: object) -> bool:
        if not isinstance(__value, TreeNode):
            return False
        return self.value == __value.value and all(
            map(lambda x: x[0] == x[1], zip(self.children, __value.children))
        )


def find_types(value):
    try:
        if isinstance(value, int):
            return TreeNode(int)

        if isinstance(value, Sequence):
            assert value, f"Empty sequences not supported {type(value)} {value}"
            return TreeNode(type(value), list(map(find_types, value)))

        raise TypeError(f"Unsupported type {type(value)} {value}")

    except RecursionError:
        raise TypeError(f"Sequence iters a sequence indefinately {type(value)} {value}")


result = find_types(test)
print(result)

copy = find_types(test)

print(result == copy)
