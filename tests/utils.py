import networkx as nx
import pytest

from python2verilog.utils.typed import typed


class Argument:
    """
    Custom argument class for pytest with unittest

    Note: replaces underscores with dash for the CLI

    Based on: https://stackoverflow.com/a/71786197

    Compatible with _pytest.config.argparsing.Argument

    """

    def __init__(self, *args, **kwargs):
        if len(args) == 1:
            self.short = None
            self.name = args[0]
            assert isinstance(self.name, str)
            assert "-" not in self.name, "Used `_` in place of `-`, but use `-` for CLI"
        elif len(args) == 2:
            assert len(args[0]) == 1, "First arg is short"
            self.short = args[0]
            self.name = args[1]
        else:
            raise ValueError("Expected 0 or 1 un-named arguments")
        self.kwargs = kwargs
        self.dashed_name = self.name.replace("_", "-")

    def add_to_parser(self, parser: pytest.Parser):
        if self.short:
            parser.addoption(f"-{self.short}", f"--{self.dashed_name}", **self.kwargs)
        else:
            parser.addoption(f"--{self.dashed_name}", **self.kwargs)

    def __repr__(self):
        return f"{__class__} {self.name}: {self.value}"
