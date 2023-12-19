"""
Lowers IR Graph to FSM
"""

import itertools
import logging
from typing import Optional

from python2verilog import ir
from python2verilog.backend.verilog import ast as ver
from python2verilog.backend.verilog.config import CodegenConfig, TestbenchConfig
from python2verilog.backend.verilog.module import Module
from python2verilog.backend.verilog.testbench import Testbench
from python2verilog.optimizer import backwards_replace
from python2verilog.utils.lines import Lines
from python2verilog.utils.typed import (
    guard,
    guard_dict,
    typed,
    typed_list,
    typed_strict,
)


class FsmBuilder:
    """
    Creates a FSM using a case block from a IR Graph
    """

    def __init__(
        self, root: ir.Node, context: ir.Context, config: Optional[CodegenConfig] = None
    ):
        # Member Vars
        if not config:
            config = CodegenConfig()
        self.visited: set[str] = set()
        self.context = context
        self.case = ver.Case(expression=context.state_var, case_items=[])
        self.root = typed_strict(root, ir.Node)
        self.config = typed_strict(config, CodegenConfig)

        # Member Funcs
        instance = itertools.count()
        self.next_unique = lambda: next(instance)

    def get_case(self) -> ver.Case:
        """
        Gets case statement/block
        """
        # Start recursion and create FSM
        self.case.case_items.append(self.new_caseitem(self.root))

        # Reverse states for readability (states are built backwards)
        self.case.case_items = list(reversed(self.case.case_items))

        return self.case

    @staticmethod
    def create_quick_done(context: ir.Context) -> ver.IfElse:
        """
        if ready:
            done = 1
            state = idle
        else:
            state = done
        """
        return ver.IfElse(
            condition=ir.UBinOp(
                ir.UnaryOp("!", context.signals.valid), "&&", context.signals.ready
            ),
            then_body=[
                ver.NonBlockingSubsitution(context.signals.done, ir.UInt(1)),
                ver.NonBlockingSubsitution(context.state_var, context.idle_state),
            ],
            else_body=[
                ver.NonBlockingSubsitution(context.state_var, context.done_state),
            ],
        )

    def new_caseitem(self, root: ir.Node):
        """
        Creates a new case item with the root's unique id as identifier
        """
        stmts = self.do_vertex(root)
        logging.debug("new caseitem %s", root.unique_id)
        item = ver.CaseItem(condition=ir.State(root.unique_id), statements=stmts)

        return item

    def do_vertex(self, vertex: ir.Node):
        """
        Processes a node
        """
        logging.debug("%s %s %s", self.do_vertex.__name__, vertex, len(self.visited))

        assert isinstance(vertex, ir.Node), str(vertex)
        self.visited.add(vertex.unique_id)

        stmts: list[ver.Statement] = []

        if isinstance(vertex, ir.AssignNode):
            stmts.append(
                ver.NonBlockingSubsitution(
                    vertex.lvalue,
                    vertex.rvalue,
                    comment=vertex.unique_id if self.config.add_debug_comments else "",
                )
            )
            stmts += self.do_edge(vertex.optimal_child)

        elif isinstance(vertex, ir.IfElseNode):
            then_body = self.do_edge(vertex.optimal_true_edge)
            else_body = self.do_edge(vertex.optimal_false_edge)
            stmts.append(
                ver.IfElse(
                    condition=vertex.condition,
                    then_body=then_body,
                    else_body=else_body,
                    comment=vertex.unique_id if self.config.add_debug_comments else "",
                )
            )
        else:
            raise TypeError(type(vertex))

        return stmts

    def do_edge(self, edge: ir.Edge):
        """
        Processes a edge
        """
        if isinstance(edge, ir.NonClockedEdge):
            return self.do_vertex(edge.optimal_child)
        if isinstance(edge, ir.ClockedEdge):
            if edge.optimal_child.unique_id not in self.visited:
                self.case.case_items.append(self.new_caseitem(edge.optimal_child))
            return [
                ver.NonBlockingSubsitution(
                    self.context.state_var, ir.State(edge.optimal_child.unique_id)
                )
            ]
        if isinstance(edge, type(None)):
            return []
        raise RuntimeError(f"{type(edge)}")
