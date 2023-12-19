"""
Verilog Codegen
"""

from typing import Optional

from python2verilog import ir
from python2verilog.backend.verilog import ast as ver
from python2verilog.backend.verilog.config import CodegenConfig, TestbenchConfig
from python2verilog.backend.verilog.fsm import FsmBuilder
from python2verilog.backend.verilog.module import Module
from python2verilog.backend.verilog.testbench import Testbench
from python2verilog.utils.typed import (
    guard,
    guard_dict,
    typed,
    typed_list,
    typed_strict,
)


class CodeGen:
    """
    Builds the Verilog ast from the context and IR
    """

    def __init__(
        self, root: ir.Node, context: ir.Context, config: Optional[CodegenConfig] = None
    ):
        if not config:
            config = CodegenConfig()
        self.context = typed_strict(context, ir.Context)
        self.config = typed_strict(config, CodegenConfig)
        root_case = FsmBuilder(root, context, config).get_case()

        for item in root_case.case_items:
            self.context.add_state_weak(
                item.condition.to_string()
            )  # change to not weak

        assert isinstance(context.done_state, ir.State)
        self.context.add_state_weak(str(context.done_state))
        self.context.add_state_weak(str(context.idle_state))
        self.case = typed_strict(root_case, ver.Case)
        self._module: Optional[ver.Module] = None
        self._testbench: Optional[ver.Module] = None

    def get_module(self):
        """
        Get Verilog module
        """
        return (
            Module(context=self.context, root=self.case)
            if self._module is None
            else self._module
        )

    def get_module_lines(self):
        """
        Get Verilog module as Lines
        """
        return self.get_module().to_lines()

    def get_module_str(self):
        """
        Get Verilog module as string
        """
        return self.get_module_lines().to_string()

    def get_testbench(self, config: TestbenchConfig):
        """
        Creates testbench with multiple test cases

        Each element of self.context.test_cases represents a single test case

        :param random_ready: whether or not to have random ready signal in the while loop
        """
        return (
            Testbench(context=self.context, config=config)
            if self._testbench is None
            else self._testbench
        )

    def get_testbench_lines(self, config: TestbenchConfig):
        """
        New Testbench as lines
        """
        return self.get_testbench(config=config).to_lines()

    def get_testbench_str(self, config: TestbenchConfig):
        """
        New testbench as str
        """
        return self.get_testbench_lines(config=config).to_string()
