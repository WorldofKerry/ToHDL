"""Utility Classes"""

from __future__ import annotations


class Lines:
    """
    A list of str that can be serialized to string with provided indents
    """

    @staticmethod
    def assert_no_newline(string: str):
        """
        Asserts no newline character in string
        """
        assert string.find("\n") == -1, "Newline in Lines: " + string

    def __init__(self, data: list[str] | str | None = None):
        if data is None:
            self.lines: list[str] = []
        elif isinstance(data, str):
            self.lines = []
            for line in data.splitlines():
                self.lines.append(line)
        elif isinstance(data, list):
            for line in data:
                assert isinstance(line, str), "Input must be a list of strings"
                self.assert_no_newline(line)
            self.lines = data
        else:
            assert False, "Invalid input type: " + str(type(data))

    def __add__(self, other: str):
        return self.add(other)

    def __str__(self):
        return self.to_string()

    def __repr__(self):
        return self.to_string()

    def __len__(self):
        return len(self.lines)

    def __getitem__(self, key: int):
        return self.lines[key]

    def __setitem__(self, key: int, value: str):
        self.lines[key] = value

    def __delitem__(self, key: int):
        del self.lines[key]

    def __rshift__(self, indent: int):
        self.indent(indent)

    @staticmethod
    def __indent(indent_amount: int, text: str):
        """
        Indent non-empty lines
        """
        if len(text.strip()) > 0:
            return Indent(indent_amount) + text
        return text

    def indent(self, indent_amount: int):
        """
        Indent all lines by amount
        """
        self.lines = [self.__indent(indent_amount, line) for line in self.lines]
        return self

    def to_string(self, indent: int = 0):
        """
        Converts all lines into a string with lines
        """
        output = ""
        for buffer in self.lines:
            output += self.__indent(indent, buffer) + "\n"
        return output

    def add(self, other: str):
        """
        Adds a new line
        """
        assert isinstance(other, str)
        self.assert_no_newline(other)
        self.lines.append(other)
        return self

    def blank(self):
        """
        Adds a blank line
        """
        self.lines.append("")
        return self

    def concat(self, other: Lines, indent: int = 0):
        """
        Concats two Lines
        """
        assert isinstance(other, Lines), f"got {type(other)} instead"
        for line in other.lines:
            self.lines.append(self.__indent(indent, line))
        return self

    @staticmethod
    def nestify(buffers: list[tuple[Lines, Lines]], indent: int = 0):
        """
        pair[0][0]
            pair[1][0]
                pair[2][0]
                pair[2][1]
            pair[1][0]
        pair[0][1]
        """
        assert isinstance(indent, int)
        for pair in buffers:
            assert isinstance(pair[0], Lines), f"{type(pair[0])} {pair[0]}"
            assert isinstance(pair[1], Lines), f"{type(pair[1])} {pair[1]}"
            assert len(pair) == 2
        lines = Lines()
        for i, buffer in enumerate(buffers):
            lines.concat(buffer[0], indent + i)
        for i, buffer in enumerate(reversed(buffers)):
            lines.concat(buffer[1], indent + len(buffers) - i - 1)
        return lines


class Indent:
    """
    Creates str instances of indentation
    """

    indentor = " " * 4

    def indentify(self, indent: int = 0) -> str:
        """
        Creates indentation
        """
        return self.indentor * indent

    def __init__(self, indent: int = 0):
        self.indent = indent

    def __add__(self, other: str):
        assert isinstance(other, str)
        return self.indentify(self.indent) + other

    def __radd__(self, other: str):
        assert isinstance(other, str)
        return other + self.indentify(self.indent)

    def __str__(self):
        return self.indentify(self.indent)


class ImplementsToLines:
    """
    A base class defining an interface for classes that need to provide a 'to_lines' method
    """

    def to_lines(self):
        """
        To Lines
        """
        raise NotImplementedError("Derived class must implement to_lines")

    def to_string(self):
        """
        To string
        """
        return str(self.to_lines())

    def __str__(self):
        return self.to_string()
