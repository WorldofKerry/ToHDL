"""
The freshest in-order generator parser
"""
from __future__ import annotations

import ast as pyast
import copy
import itertools
import logging
from typing import Collection, Iterable, Optional, TypeVar, cast

from typing_extensions import TypeAlias

from python2verilog import ir
from python2verilog.exceptions import StaticTypingError, UnsupportedSyntaxError
from python2verilog.utils.typed import guard, typed_list, typed_strict


class Function:
    """
    Parses python functions and generator functions
    """

    # (head_node, edge_tails) representing head of a tree and all leaf edges
    ParseResult: TypeAlias = tuple[ir.Node, list[ir.Edge]]

    def __init__(self, context: ir.Context, prefix: str = "") -> None:
        self.__context = copy.deepcopy(context)
        self.__context.prefix = prefix
        self.__context.refresh_input_output_vars()  # Update to use prefix
        self.__head_and_tails: Optional[tuple[ir.Node, list[ir.Edge]]] = None

    def parse_function(self) -> tuple[ir.Context, ir.Node]:
        """
        Parses function as an independent unit.

        Cache result if called multiple times.

        :return: context, head
        """
        return self.parse_inline()[:2]

    def parse_inline(self) -> tuple[ir.Context, ir.Node, list[ir.Edge]]:
        """
        Parses function for inlining.

        Caches result if called multiple times.

        :return: context, head, tails
        """
        if self.__head_and_tails:
            return copy.deepcopy(self.__context), *self.__head_and_tails

        breaks: list[ir.Edge] = []
        continues: list[ir.Edge] = []
        body_head, prev_tails = self._parse_stmts(
            self.__context.py_ast.body,
            prefix=f"_state{self.__context.prefix}",
            breaks=breaks,
            continues=continues,
        )
        self.__context.entry_state = ir.State(body_head.unique_id)

        assert len(breaks) == 0
        assert len(continues) == 0

        for tail in prev_tails:
            tail.child = self._create_done(prefix="_state_done")

        self.__head_and_tails = body_head, prev_tails
        return copy.deepcopy(self.__context), *self.__head_and_tails

    def _create_done(self, prefix: str) -> ir.Node:
        """
        Creates the done nodes

        Signals that module is done, happens in one clock cycle
        """
        left_hand_sides = [
            self.__context.signals.done,
            self.__context.signals.valid,
            self.__context.state_var,
        ]
        right_hand_sides = [ir.UInt(1), ir.UInt(1), self.__context.idle_state]
        head, _tail = self._weave_nonclocked_edges(
            self._create_assign_nodes(left_hand_sides, right_hand_sides, prefix=prefix),
            prefix=f"{prefix}_e",
            last_edge=False,
        )
        return head

    def _parse_stmt(
        self,
        stmt: pyast.AST,
        breaks: list[ir.Edge],
        continues: list[ir.Edge],
        prefix: str,
    ) -> ParseResult:
        """
        Parses a statement

        :param stmt: statement to-be-parsed
        :param breaks: list of edges that connect to end of break stmt
        :param continues: where to connect continue edges
        """
        if isinstance(stmt, pyast.Assign):
            return self._parse_assign(assign=stmt, prefix=prefix)
        if isinstance(stmt, pyast.Yield):
            return self._parse_yield(yiel=stmt, prefix=prefix)
        if isinstance(stmt, pyast.Expr):
            return self._parse_stmt(
                stmt=stmt.value, breaks=breaks, continues=continues, prefix=prefix
            )
        if isinstance(stmt, pyast.While):
            return self._parse_while(whil=stmt, prefix=prefix)
        if isinstance(stmt, pyast.Break):
            nothing = ir.AssignNode(
                unique_id=prefix,
                name="break",
                lvalue=self.__context.state_var,
                rvalue=self.__context.state_var,
                child=ir.ClockedEdge(
                    unique_id=f"{prefix}_e",
                ),
            )
            assert guard(nothing.child, ir.Edge)
            breaks.append(nothing.child)
            return nothing, []
        if isinstance(stmt, pyast.If):
            return self._parse_ifelse(
                ifelse=stmt, prefix=prefix, breaks=breaks, continues=continues
            )
        if isinstance(stmt, pyast.For):
            return self._parse_for(stmt=stmt, prefix=prefix)
        if isinstance(stmt, pyast.AugAssign):
            assert isinstance(
                stmt.target, pyast.Name
            ), "Error: only supports single target"
            edge = ir.ClockedEdge(unique_id=f"{prefix}_e")
            lvalue = self._parse_expression(stmt.target)
            assert guard(lvalue, ir.Var)
            cur_node = ir.AssignNode(
                unique_id=prefix,
                lvalue=lvalue,
                rvalue=self._parse_expression(
                    pyast.BinOp(stmt.target, stmt.op, stmt.value)
                ),
                child=edge,
            )
            return cur_node, [edge]
        if isinstance(stmt, pyast.Continue):
            nothing = ir.AssignNode(
                unique_id=prefix,
                name="break",
                lvalue=self.__context.state_var,
                rvalue=self.__context.state_var,
                child=ir.ClockedEdge(
                    unique_id=f"{prefix}_e",
                ),
            )
            assert guard(nothing.child, ir.Edge)
            continues.append(nothing.child)
            return nothing, []
        if isinstance(stmt, pyast.Return):
            return self._parse_return(ret=stmt, prefix=prefix)
        if isinstance(stmt, (pyast.Call, pyast.Constant)):
            if isinstance(stmt, pyast.Constant):
                # Probably a triple-quote comment
                assert guard(stmt.value, str)
            else:
                logging.info(
                    "Ignored function call %s",
                    pyast.dump(stmt, include_attributes=True),
                )
            dummy = ir.AssignNode(
                unique_id=prefix,
                name="dummy",
                lvalue=self.__context.state_var,
                rvalue=self.__context.state_var,
                child=ir.ClockedEdge(
                    unique_id=f"{prefix}_e",
                ),
            )
            assert guard(dummy.child, ir.Edge)
            return dummy, [dummy.child]

        raise UnsupportedSyntaxError.from_pyast(stmt, self.__context.name)

    def _parse_return(self, ret: pyast.Return, prefix: str) -> ParseResult:
        """
        Parses return
        """
        if ret.value is None and self.__context.is_generator:
            done = self._create_done(prefix=prefix)
            return done, []
        assert not self.__context.is_generator

        try:
            if isinstance(ret.value, pyast.Tuple):
                stmts = [self._parse_expression(c) for c in ret.value.elts]
            elif isinstance(ret.value, pyast.expr):
                stmts = [self._parse_expression(ret.value)]
            else:
                raise UnsupportedSyntaxError.from_pyast(ret, self.__context.name)
        except Exception as e:
            raise UnsupportedSyntaxError.from_pyast(ret, self.__context.name) from e

        self.__context.validate()
        head, tail = self._weave_nonclocked_edges(
            self._create_assign_nodes(self.__context.output_vars, stmts, prefix),
            f"{prefix}_e",
        )
        tail.edge = ir.ClockedEdge(unique_id=f"{prefix}_last_e")
        return head, [tail.edge]

    def _parse_stmts(
        self,
        stmts: list[pyast.stmt],
        prefix: str,
        breaks: list[ir.Edge],
        continues: list[ir.Edge],
    ) -> ParseResult:
        """
        A helper for parsing statements (blocks of code)

        :return: (head, breaks, continues, ends)
        - head is the head of this body (first node in body)
        - ends are edges come from end of body (what to do after body)
        """
        body_head: Optional[ir.Node] = None
        prev_tails: Optional[list[ir.Edge]] = None
        counter = itertools.count()

        for stmt in stmts:
            head_node, tail_edges = self._parse_stmt(
                stmt=stmt,
                breaks=breaks,
                continues=continues,
                prefix=f"{prefix}{next(counter)}",
            )
            if not body_head:
                body_head = head_node
            if prev_tails is not None:
                for tail in prev_tails:
                    tail.child = head_node
            prev_tails = typed_list(tail_edges, ir.Edge)
            assert guard(head_node, ir.Node)

        assert prev_tails is not None
        assert body_head is not None
        return body_head, prev_tails

    def _parse_assign_to_call(self, assign: pyast.Assign, prefix: str) -> ParseResult:
        """
        instance = func(args, ...)
        """
        assert isinstance(assign.value, pyast.Call)
        assert len(assign.targets) == 1
        target = assign.targets[0]
        assert isinstance(target, pyast.Name)

        def get_func_call_names():
            """
            :return: target_name, func_name
            """
            # Figure out target name
            target_name = target.id

            # Figure out func being called
            assert guard(assign.value, pyast.Call)
            func = assign.value.func
            assert guard(func, pyast.Name)
            func_name = func.id

            return target_name, func_name

        target_name, func_name = get_func_call_names()

        # Get context of generator function being called
        callee_cxt = self.__context.namespace[func_name]

        if callee_cxt.is_generator:
            return self._parse_gen_call(
                call_args=assign.value.args,
                callee_cxt=callee_cxt,
                target_name=target_name,
                prefix=prefix,
            )
        return self._parse_func_call(
            call_args=assign.value.args,
            callee_cxt=callee_cxt,
            targets=assign.targets,
            _target_name=target_name,
            prefix=prefix,
        )

    def _parse_func_call(
        self,
        call_args: list[pyast.expr],
        targets: list[pyast.expr],
        callee_cxt: ir.Context,
        _target_name: str,
        prefix: str,
    ) -> ParseResult:
        """
        Responsible for parsing calls to non-generator functions.

        Implemented as an inline (no external unit).
        """
        callee_cxt, body_head, prev_tails = Function(
            callee_cxt, prefix=f"{prefix}_"
        ).parse_inline()

        arguments = list(map(self._parse_expression, call_args))

        inputs_head, tail = self._weave_nonclocked_edges(
            self._create_assign_nodes(
                callee_cxt.input_vars,
                arguments,
                prefix=f"{prefix}_inputs",
            ),
            prefix=f"{prefix}_inputs_e",
        )
        tail.edge = ir.ClockedEdge(unique_id=f"{prefix}_inputs_last_e", child=body_head)

        results = typed_list(list(map(self._parse_expression, targets)), ir.Var)
        head, tail = self._weave_nonclocked_edges(
            self._create_assign_nodes(
                results,
                callee_cxt.output_vars,
                prefix=f"{prefix}_outputs",
            ),
            prefix=f"{prefix}_outputs_e",
        )
        tail.edge = ir.ClockedEdge(unique_id=f"{prefix}_outputs_last_e")
        for prev_tail in prev_tails:
            prev_tail.child = head

        for var in (
            *callee_cxt.input_vars,
            *callee_cxt.output_vars,
            *callee_cxt.local_vars,
            *results,
        ):
            self.__context.add_special_local_var(var)

        return inputs_head, [tail.edge]

    def _parse_gen_call(
        self,
        call_args: list[pyast.expr],
        callee_cxt: ir.Context,
        target_name: str,
        prefix: str,
    ) -> ParseResult:
        """
        Responsible for parsing calls of generator functions
        """
        inst = callee_cxt.create_generator_instance(target_name)
        self.__context.generator_instances[target_name] = inst

        def unique_node_gen():
            counter = 0
            while True:
                yield f"{prefix}_call_{counter}"
                counter += 1

        def unique_edge_gen():
            counter = 0
            while True:
                yield f"{prefix}_call_{counter}_e"
                counter += 1

        unique_node = unique_node_gen()
        unique_edge = unique_edge_gen()

        # Nessessary for exclusivity
        head = ir.AssignNode(
            unique_id=next(unique_node),
            lvalue=inst.signals.ready,
            rvalue=ir.UInt(0),
        )

        node: ir.BasicElement = head

        node.child = ir.NonClockedEdge(
            unique_id=next(unique_edge),
        )
        node = node.child

        node.child = ir.AssignNode(
            unique_id=next(unique_node),
            lvalue=inst.signals.start,
            rvalue=ir.UInt(1),
        )
        node = node.child
        node.child = ir.NonClockedEdge(
            unique_id=next(unique_edge),
        )
        node = node.child

        arguments = list(map(self._parse_expression, call_args))

        assert len(arguments) == len(inst.inputs)

        assign_head, tail = self._weave_nonclocked_edges(
            self._create_assign_nodes(inst.inputs, arguments, prefix),
            f"{prefix}_e",
        )
        node.child = assign_head
        tail.child = ir.ClockedEdge(
            unique_id=next(unique_edge),
        )
        return head, [tail.child]

    def _name_to_var(self, name: pyast.expr) -> ir.Var:
        """
        Converts a pyast.Name to a ir.Var using its id
        """
        assert isinstance(name, pyast.Name)
        return self.__context.make_var(name.id)

    def _parse_for(self, stmt: pyast.For, prefix: str) -> ParseResult:
        """
        For ... in ...:
        """

        assert not stmt.orelse, "for-else statements not supported"
        target = stmt.iter
        if not isinstance(target, pyast.Name):
            return self._parse_for_gen_call(stmt, prefix)
        return self._parse_for_gen_instance(stmt, prefix)

    def _get_func_call_name(self, call: pyast.Call) -> str:
        assert guard(call.func, pyast.Name)
        return call.func.id

    def _get_single_target_name(self, targets: list[pyast.expr]) -> str:
        assert len(targets) == 1
        assert guard(targets[0], pyast.Name)
        return targets[0].id

    def _get_target_vars(self, target: pyast.expr) -> list[ir.Var]:
        """
        Converts a target into variables

        (a, b, c) -> a, b, c
        a -> a
        """
        if not isinstance(target, pyast.Tuple):
            assert isinstance(target, pyast.Name)
            outputs = [self.__context.make_var(target.id)]
        else:
            outputs = list(map(self._name_to_var, target.elts))
        return outputs

    def _parse_for_gen_call(self, stmt: pyast.For, prefix: str) -> ParseResult:
        """
        For <targets> in <gen_call>:

        where gen_call is a function that returns a generator instance
        """

        # deal with `... in <gen_call>`
        assert guard(stmt.iter, pyast.Call)
        gen_cxt = self.__context.namespace[self._get_func_call_name(stmt.iter)]

        mangled_name = f"nested{stmt.col_offset}"  # consider nested for loops

        call_head, call_tails = self._parse_gen_call(
            call_args=stmt.iter.args,
            target_name=mangled_name,
            prefix=prefix,
            callee_cxt=gen_cxt,
        )

        # deal with `for <targets> ...` and body
        inst = self.__context.generator_instances[mangled_name]
        targets = self._get_target_vars(stmt.target)

        body_head, body_tails = self._parse_for_target_and_body(
            inst=inst,
            prefix=prefix,
            body=stmt.body,
            targets=targets,
        )

        assert len(call_tails) == 1
        call_tails[0].child = body_head

        return call_head, body_tails

    def _parse_for_gen_instance(self, stmt: pyast.For, prefix: str) -> ParseResult:
        """
        For <targets> in <instance>:

        where instance is a generator instance
        """
        iterable = stmt.iter
        assert guard(iterable, pyast.Name)
        assert (
            iterable.id in self.__context.generator_instances
        ), f"No iterator instance {self.__context.generator_instances}"
        inst = self.__context.generator_instances[iterable.id]

        targets = self._get_target_vars(stmt.target)
        assert len(targets) == len(inst.outputs), f"{targets} {inst.outputs}"

        return self._parse_for_target_and_body(
            inst=inst, prefix=prefix, body=stmt.body, targets=targets
        )

    def _parse_for_target_and_body(
        self,
        targets: list[ir.Var],
        inst: ir.Instance,
        body: list[pyast.stmt],
        prefix: str,
    ) -> ParseResult:
        """
        Responsible for parsing the target and body of the for loop

        for <target> ... <inst>:
            <body>
        """
        # pylint: disable=too-many-locals

        def gen_unique_node():
            counter = 0
            while True:
                yield f"{prefix}_for_{counter}"
                counter += 1

        def gen_unique_edge():
            counter = 0
            while True:
                yield f"{prefix}_for_{counter}_e"
                counter += 1

        unique_node = gen_unique_node()
        unique_edge = gen_unique_edge()

        to_done = ir.ClockedEdge(next(unique_edge))  # goto end of for
        to_body = ir.ClockedEdge(next(unique_edge))  # goto body of for
        to_wait = ir.ClockedEdge(next(unique_edge))  # repeat check

        to_inner = ir.NonClockedEdge(
            next(unique_edge)
        )  # to inner ifelse (done or iterate)

        head = ir.IfElseNode(
            unique_id=next(unique_node),
            condition=ir.UBinOp(inst.signals.ready, "&&", inst.signals.valid),
            true_edge=to_inner,
            false_edge=to_wait,
        )

        # repeat check
        to_wait.child = ir.IfElseNode(
            unique_id=next(unique_node),
            condition=ir.UBinOp(
                self.__context.signals.ready,
                "||",
                ir.UnaryOp("!", self.__context.signals.valid),
            ),
            true_edge=ir.NonClockedEdge(
                next(unique_edge),
                child=ir.AssignNode(
                    unique_id=next(unique_node),
                    lvalue=inst.signals.ready,
                    rvalue=ir.UInt(1),
                    child=ir.ClockedEdge(next(unique_edge), child=head),
                ),
            ),
            false_edge=ir.ClockedEdge(next(unique_edge), child=head),
        )

        # inner ifelse
        to_capture = ir.NonClockedEdge(next(unique_edge))
        to_inner.child = ir.AssignNode(
            unique_id=next(unique_node),
            lvalue=inst.signals.ready,
            rvalue=ir.UInt(0),
            child=ir.NonClockedEdge(
                next(unique_edge),
                child=ir.IfElseNode(
                    unique_id=next(unique_node),
                    condition=inst.signals.done,
                    true_edge=to_done,
                    false_edge=to_capture,
                ),
            ),
        )

        def create_capture_output_nodes():
            dummy = ir.NonClockedEdge(next(unique_edge))
            prev_edge: ir.Edge = dummy
            for caller, callee in zip(targets, inst.outputs):
                assert guard(prev_edge, ir.Edge)
                prev_assign = ir.AssignNode(
                    unique_id=next(unique_node),
                    lvalue=caller,
                    rvalue=callee,
                    child=ir.NonClockedEdge(next(unique_edge)),
                )
                prev_edge.child = prev_assign
                assert guard(prev_assign.child, ir.Edge)
                prev_edge = prev_assign.child
                self.__context.add_local_var(caller)

            assert guard(prev_assign, ir.AssignNode)
            prev_assign.child = to_body
            return dummy.child

        capture_node = create_capture_output_nodes()
        assert guard(capture_node, ir.AssignNode)
        to_capture.child = capture_node

        breaks: list[ir.Edge] = []
        continues: list[ir.Edge] = []
        body_node, ends = self._parse_stmts(
            stmts=body,
            prefix=f"{prefix}_for_body",
            breaks=breaks,
            continues=continues,
        )
        to_body.child = body_node

        for cont in continues:
            cont.child = head
        for end in ends:
            end.child = head

        return head, [to_done, *breaks]

    def _parse_while(self, whil: pyast.While, prefix: str) -> ParseResult:
        assert not whil.orelse, "while-else statements not supported"
        breaks: list[ir.Edge] = []
        continues: list[ir.Edge] = []
        body_head, ends = self._parse_stmts(
            stmts=whil.body,
            prefix=f"{prefix}_while",
            breaks=breaks,
            continues=continues,
        )
        done_edge = ir.ClockedEdge(unique_id=f"{prefix}_done_e")
        while_head = ir.IfElseNode(
            unique_id=f"{prefix}_while",
            condition=self._parse_expression(whil.test),
            true_edge=ir.ClockedEdge(unique_id=f"{prefix}_body_e", child=body_head),
            false_edge=done_edge,
        )
        for cont in continues:
            cont.child = while_head
        for end in ends:
            end.child = while_head
        return while_head, [done_edge, *breaks]

    def _parse_ifelse(
        self,
        ifelse: pyast.If,
        prefix: str,
        breaks: list[ir.Edge],
        continues: list[ir.Edge],
    ) -> ParseResult:
        then_head, then_ends = self._parse_stmts(
            stmts=ifelse.body,
            prefix=f"{prefix}_then",
            breaks=breaks,
            continues=continues,
        )
        to_then = ir.NonClockedEdge(unique_id=f"{prefix}_then_e", child=then_head)

        if ifelse.orelse:
            else_head, else_ends = self._parse_stmts(
                stmts=ifelse.orelse,
                prefix=f"{prefix}_else",
                breaks=breaks,
                continues=continues,
            )
            to_else = ir.NonClockedEdge(unique_id=f"{prefix}_else_e", child=else_head)
            ret = ir.IfElseNode(
                unique_id=prefix,
                condition=self._parse_expression(ifelse.test),
                true_edge=to_then,
                false_edge=to_else,
            )
            return ret, [*then_ends, *else_ends]

        # No else
        to_else = ir.NonClockedEdge(unique_id=f"{prefix}_else_e")
        ret = ir.IfElseNode(
            unique_id=prefix,
            condition=self._parse_expression(ifelse.test),
            true_edge=to_then,
            false_edge=to_else,
        )
        return ret, [*then_ends, to_else]

    @staticmethod
    def pprint(elem: ir.Element):
        """
        Get node str
        """
        return str(list(elem.visit_nonclocked()))

    def _parse_yield(self, yiel: pyast.Yield, prefix: str) -> ParseResult:
        """
        Parse yield
        yield <value>
        """
        if isinstance(yiel.value, pyast.Tuple):
            stmts = [self._parse_expression(c) for c in yiel.value.elts]
        elif isinstance(yiel.value, pyast.expr):
            stmts = [self._parse_expression(yiel.value)]
        else:
            raise TypeError(f"Expected tuple {type(yiel.value)} {pyast.dump(yiel)}")

        self.__context.validate()
        head, tail = self._weave_nonclocked_edges(
            self._create_assign_nodes(self.__context.output_vars, stmts, prefix),
            f"{prefix}_e",
        )
        tail_edge: ir.Edge = tail.edge  # get last nonclocked edge
        tail_edge.child = ir.AssignNode(
            unique_id=f"{prefix}_valid",
            lvalue=self.__context.signals.valid,
            rvalue=ir.UInt(1),
            child=ir.ClockedEdge(unique_id=f"{prefix}_e"),
        )
        return head, [tail_edge.child.edge]

    def _target_value_visitor(self, target: pyast.expr, value: pyast.expr):
        """
        Takes two trees, ensures they're idential,
        then yields the target/value pairs
        """
        if isinstance(value, pyast.Tuple):
            if guard(target, pyast.Name):
                raise TypeError(
                    f"Attempted to assign {target.id} = {pyast.unparse(value)}."
                    " Currently variables can only containt ints."
                )
            assert guard(target, pyast.Tuple)
            assert len(target.elts) == len(value.elts)
            for t, v in zip(target.elts, value.elts):
                yield from self._target_value_visitor(t, v)
        elif isinstance(target, pyast.Name):
            var = self.__context.make_var(target.id)
            self.__context.add_local_var(var)
            yield (var, self._parse_expression(value))
        else:
            raise TypeError(f"{pyast.dump(target)} {pyast.dump(value)}")

    @staticmethod
    def _create_assign_nodes(
        variables: Collection[ir.Var],
        exprs: Collection[ir.Expression],
        prefix: str,
    ) -> Iterable[ir.AssignNode]:
        """
        Create assign nodes from variables and expressions
        """
        assert len(variables) == len(exprs), f"{variables} {exprs}"
        counters = itertools.count()
        for var, expr, counter in zip(variables, exprs, counters):
            yield ir.AssignNode(
                unique_id=f"{prefix}_assign{counter}", lvalue=var, rvalue=expr
            )

    _BasicNodeType = TypeVar("_BasicNodeType", bound=ir.BasicNode)

    @staticmethod
    def _weave_nonclocked_edges(
        nodes: Iterable[_BasicNodeType], prefix: str, last_edge: bool = True
    ) -> tuple[_BasicNodeType, _BasicNodeType]:
        """
        Weaves nodes with nonclocked edges.

        :param last_edge: if True, include last nonclocked edge
        :return: (first basic node, last basic node)
        """
        counters = itertools.count()
        head: Optional[ir.BasicNode] = None
        prev: Optional[ir.BasicElement] = None
        node: ir.BasicElement = ir.BasicElement(unique_id="DUMMY")
        for node, counter in zip(nodes, counters):
            if not head:
                head = node
            if prev:
                prev.child = node
            node.child = ir.NonClockedEdge(
                unique_id=f"{prefix}_{counter}",
            )
            prev = node.child
        if not last_edge:
            node._child = None  # pylint: disable=protected-access
        return cast(Function._BasicNodeType, head), node

    def _parse_assign(self, assign: pyast.Assign, prefix: str) -> ParseResult:
        """
        target0 [= target1] ... = value;

        Example (single target):
        a, b = b, a + b

        target value visitor
        a -> b
        b -> a + b

        create assign nodes
        a = b
        b = a + b

        weave
        a = b
          -> b = a + b
            -> None
        """
        if isinstance(assign.value, pyast.Call):
            return self._parse_assign_to_call(assign=assign, prefix=prefix)

        assert len(assign.targets) == 1
        targets, values = zip(
            *self._target_value_visitor(assign.targets[0], assign.value)
        )

        head, tail = self._weave_nonclocked_edges(
            self._create_assign_nodes(targets, values, prefix),
            f"{prefix}_e",
        )
        logging.debug("Assign Head %s", list(head.visit_nonclocked()))

        tail.child = ir.ClockedEdge(
            unique_id=f"{prefix}",
        )
        return head, [tail.child]

    def _parse_expression(self, expr: pyast.expr) -> ir.Expression:
        """
        <expression> (e.g. constant, name, subscript, etc., those that return a value)
        """
        if isinstance(expr, pyast.Constant):
            return ir.Int(expr.value)
        if isinstance(expr, pyast.Name):
            return self.__context.make_var(expr.id)
        if isinstance(expr, pyast.Subscript):
            return self._parse_subscript(expr)
        if isinstance(expr, pyast.BinOp):
            return self._parse_binop(expr)
        if isinstance(expr, pyast.UnaryOp):
            if isinstance(expr.op, pyast.USub):
                return ir.UnaryOp("-", self._parse_expression(expr.operand))
        if isinstance(expr, pyast.Compare):
            return self._parse_compare(expr)
        if isinstance(expr, pyast.BoolOp):
            if isinstance(expr.op, pyast.And):
                return ir.UBinOp(
                    self._parse_expression(expr.values[0]),
                    "&&",
                    self._parse_expression(expr.values[1]),
                )
        raise UnsupportedSyntaxError.from_pyast(expr, self.__context.name)

    def _parse_subscript(self, node: pyast.Subscript) -> ir.Expression:
        """
        <value>[<slice>]
        Note: built from right to left, e.g. [z] -> [y][z] -> [x][y][z] -> matrix[x][y][z]
        """
        return ir.Expression(
            f"{self._parse_expression(node.value).to_string()}\
                [{self._parse_expression(node.slice).to_string()}]"
        )

    def _parse_compare(self, node: pyast.Compare) -> ir.UBinOp:
        """
        <left> <op> <comparators>
        """
        assert len(node.ops) == 1
        assert len(node.comparators) == 1

        if isinstance(node.ops[0], pyast.Lt):
            return ir.LessThan(
                self._parse_expression(node.left),
                self._parse_expression(node.comparators[0]),
            )
        if isinstance(node.ops[0], pyast.LtE):
            operator = "<="
        elif isinstance(node.ops[0], pyast.Gt):
            operator = ">"
        elif isinstance(node.ops[0], pyast.GtE):
            operator = ">="
        elif isinstance(node.ops[0], pyast.NotEq):
            operator = "!=="
        elif isinstance(node.ops[0], pyast.Eq):
            operator = "==="
        else:
            raise UnsupportedSyntaxError(
                f"Unsupported operator {pyast.dump(node.ops[0])}"
            )
        return ir.UBinOp(
            left=self._parse_expression(node.left),
            oper=operator,
            right=self._parse_expression(node.comparators[0]),
        )

    def _parse_binop(self, expr: pyast.BinOp) -> ir.Expression:
        """
        <left> <op> <right>

        With special case for floor division
        """
        if isinstance(expr.op, pyast.Add):
            return ir.Add(
                self._parse_expression(expr.left), self._parse_expression(expr.right)
            )
        if isinstance(expr.op, pyast.Sub):
            return ir.Sub(
                self._parse_expression(expr.left), self._parse_expression(expr.right)
            )

        if isinstance(expr.op, pyast.Mult):
            return ir.Mul(
                self._parse_expression(expr.left), self._parse_expression(expr.right)
            )

        if isinstance(expr.op, pyast.FloorDiv):
            left = self._parse_expression(expr.left)
            right = self._parse_expression(expr.right)
            return ir.FloorDiv(left, right)
        if isinstance(expr.op, pyast.Mod):
            left = self._parse_expression(expr.left)
            right = self._parse_expression(expr.right)
            return ir.Mod(left, right)
        raise UnsupportedSyntaxError(f"Unsupported binop `{pyast.dump(expr.op)}")
