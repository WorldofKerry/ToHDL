"""
Peekable counter
"""

from typing import Iterable


class PeekCounter(Iterable[int]):
    """
    Peekable counter

    Based on itertools.count
    """

    def __init__(self, start: int = 0) -> None:
        self.state = start

    def __next__(self):
        return self.next()

    def __iter__(self):
        return self

    def next(self) -> int:
        """
        Gets next value in count
        """
        self.state += 1
        return self.state

    def peek(self) -> int:
        """
        Peeks next value in count
        """
        return self.state + 1
