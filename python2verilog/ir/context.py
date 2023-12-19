"""
The context of a generator

Includes the function, its representation and its usage information
"""
from __future__ import annotations

import ast
import copy
import logging
from dataclasses import dataclass, field
from itertools import zip_longest
from types import FunctionType
from typing import Any, Optional

from python2verilog.api.modes import Modes
from python2verilog.exceptions import StaticTypingError, TypeInferenceError
from python2verilog.ir.expressions import ExclusiveVar, State, Var
from python2verilog.ir.instance import Instance
from python2verilog.ir.signals import ProtocolSignals
from python2verilog.utils.generics import GenericReprAndStr
from python2verilog.utils.typed import guard, typed, typed_list, typed_strict


@dataclass
class Context(GenericReprAndStr):
    """
    Context needed by the Intermediate Representation
    E.g. variables, I/O, parameters, localparam
    """

    # pylint: disable=too-many-instance-attributes, too-many-public-methods
    _frozen: bool = False  # Make immutable

    name: str = ""
    testbench_suffix: str = "_tb"
    is_generator: bool = False
    prefix: str = ""

    test_cases: list[tuple[int, ...]] = field(default_factory=list)

    py_func: Optional[FunctionType] = None
    py_string: Optional[str] = None
    _py_ast: Optional[ast.FunctionDef] = None

    input_types: Optional[list[type[Any]]] = None
    output_types: Optional[list[type[Any]]] = None

    optimization_level: int = -1

    mode: Modes = Modes.NO_WRITE

    _local_vars: list[Var] = field(default_factory=list)
    _input_vars: Optional[list[Var]] = None
    _output_vars: Optional[list[ExclusiveVar]] = None
    _states: set[str] = field(default_factory=set)

    signals: ProtocolSignals = ProtocolSignals()

    state_var: Var = Var("state")

    _done_state: State = State("_state_done")
    idle_state: State = State("_state_idle")
    _entry_state: Optional[State] = None

    # Function calls
    namespace: dict[str, Context] = field(default_factory=dict)  # callable functions
    generator_instances: dict[str, Instance] = field(
        default_factory=dict
    )  # generator instances

    def freeze(self):
        """
        Freeze this context to be immutable
        """
        self._frozen = True

    def __setattr__(self, attr, value):
        if self._frozen:
            raise AttributeError("Frozen")
        return super().__setattr__(attr, value)

    @classmethod
    def empty_valid(cls):
        """
        Creates an empty but valid context for testing purposes
        """
        cxt = cls()
        cxt.input_types = []
        cxt.input_vars = []
        cxt.output_types = []
        cxt._output_vars = []
        return cxt

    @property
    def testbench_name(self) -> str:
        """
        Returns test bench module name in the generated verilog
        """
        return f"{self.name}{self.testbench_suffix}"

    def _use_input_type_hints(self):
        """
        Uses input type hints
        """

        def input_mapper(arg: ast.arg) -> type[Any]:
            """
            Maps a string annotation id to type
            """
            assert arg.annotation, f"No type hint annotation on argument {arg.arg}"
            assert isinstance(arg.annotation, ast.Name)
            if arg.annotation.id == "int":
                return type(0)
            raise TypeError(f"{ast.dump(arg)}")

        logging.info("Using type hints of %s for input types", self.name)
        input_args: list[ast.arg] = self.py_ast.args.args
        assert isinstance(input_args, list), f"{ast.dump(self.py_ast)}"
        try:
            self.input_types = list(map(input_mapper, input_args))
        except Exception as e:
            raise TypeInferenceError(self.name) from e

    def _use_output_type_hints(self):
        """
        Use output type hints
        """

        def output_mapper(arg: ast.Name) -> type[Any]:
            """
            Maps a string annotation id to type
            """
            assert arg, "No return type hint annotation"
            if arg.id == "int":
                return type(0)
            raise TypeError(f"{ast.dump(arg)}")

        logging.info("Using type hints of %s for return types", self.name)
        output_args: list[ast.Name]
        if isinstance(self.py_ast.returns, ast.Subscript):
            assert isinstance(self.py_ast.returns.slice, ast.Tuple)
            output_args = []
            for elt in self.py_ast.returns.slice.elts:
                assert guard(elt, ast.Name)
                output_args.append(elt)
        else:
            output_args = [self.py_ast.returns]
        assert isinstance(output_args, list), f"{ast.dump(self.py_ast)}"
        try:
            self.output_types = list(map(output_mapper, output_args))
        except Exception as e:
            raise TypeInferenceError(self.name) from e
        self.default_output_vars()

    def validate(self):
        """
        Validates that all fields of context are populated.

        Populate input & output types from type hints if they're not determined

        :return: self
        """
        assert isinstance(self.py_ast, ast.FunctionDef), self
        assert isinstance(self.py_func, FunctionType), self

        if self.input_types is None:
            self._use_input_type_hints()

        assert isinstance(self.input_types, list), self
        assert isinstance(self.input_vars, list), self

        if self.output_types is None:
            self._use_output_type_hints()

        assert isinstance(self.output_types, list), self
        assert isinstance(self._output_vars, list), self

        assert isinstance(self.optimization_level, int), self
        assert self.optimization_level >= 0, f"{self.optimization_level} {self.name}"

        if self._entry_state:
            assert str(self.entry_state) in self.states, self

        for value in self.signals.variable_values():
            assert typed(value, Var)

        return self

    @property
    def py_ast(self):
        """
        Python ast node rooted at function
        """
        assert isinstance(self._py_ast, ast.FunctionDef)
        return copy.deepcopy(self._py_ast)

    @py_ast.setter
    def py_ast(self, other: ast.FunctionDef):
        assert isinstance(other, ast.FunctionDef)
        self._py_ast = other

    @property
    def entry_state(self):
        """
        The first state that does work in the graph representation
        """
        assert isinstance(self._entry_state, State), self
        return copy.deepcopy(self._entry_state)

    @entry_state.setter
    def entry_state(self, other: State):
        assert isinstance(other, State)
        self._entry_state = other

    @property
    def done_state(self):
        """
        The ready state
        """
        assert isinstance(self._done_state, State), self
        return copy.deepcopy(self._done_state)

    @done_state.setter
    def done_state(self, other: State):
        assert isinstance(other, State)
        self._done_state = other

    @property
    def input_vars(self) -> list[Var]:
        """
        Input variables
        """
        assert guard(self._input_vars, list)
        return copy.deepcopy(self._input_vars)

    @input_vars.setter
    def input_vars(self, other: list[Var]):
        self._input_vars = typed_list(other, Var)

    @property
    def output_vars(self) -> list[Var]:
        """
        Output variables
        """
        assert guard(self._output_vars, list), f"Unknown output variables {self}"
        return copy.deepcopy(self._output_vars)

    def default_output_vars(self):
        """
        Sets own output vars to default based on number of output variables
        """
        assert self.output_types is not None
        self._output_vars = [
            ExclusiveVar(f"{self.prefix}_output_{i}")
            for i in range(len(self.output_types))
        ]

    def refresh_input_output_vars(self):
        """
        Update input vars with prefix
        """
        self._input_vars = list(
            map(lambda x: self.make_var(x.py_name), self.input_vars),
        )
        self.default_output_vars()

    @property
    def local_vars(self) -> list[Var]:
        """
        Gets local variables
        """
        return copy.deepcopy(self._local_vars)

    def add_local_var(self, var: Var):
        """
        Appends to global vars with restrictions.

        Good for appending Vars created from Python source
        """
        assert var.py_name.startswith(self.prefix)
        prefixless = var.py_name[len(self.prefix) :]
        if len(prefixless) != 1:
            # A local var named "_" with no additional characters is ok
            assert not prefixless.startswith("_"), (
                'Local variables beginning with "_" '
                f"with more than one character are reserved {var.py_name}"
            )
        return self.add_special_local_var(var)

    def add_special_local_var(self, var: Var):
        """
        Appends to local vars without restrictions.

        Good for appending Vars created internally (e.g. inline function calls)
        """
        if var.py_name in self.generator_instances:
            raise StaticTypingError(
                f"Variable `{var.py_name}` changed type from generator"
                f" instance to another type in {self.name}"
            )
        if var not in (*self._local_vars, *self.input_vars, *self.output_vars):
            self._local_vars.append(typed_strict(var, Var))

    @property
    def states(self):
        """
        State variables
        """
        return copy.deepcopy(self._states)

    def add_state(self, name: str):
        """
        Add a state, making sure no pre-existing state what that name exists
        """
        if name in self._states:
            raise RuntimeError(f"Attempting to add {name} when it already exists")
        self.add_state_weak(name)

    def add_state_weak(self, name: str):
        """
        Add a state
        """
        assert isinstance(name, str)
        self._states.add(name)

    def _check_types(
        self, expected_types: list[type[Any]], actual_values: list[type[Any]]
    ):
        for expected, actual in zip_longest(
            expected_types, actual_values, fillvalue=type(None)
        ):
            assert isinstance(
                actual, expected
            ), f"Expected {expected}, got {type(actual)}, with value {actual}, \
                in call to {self.name}"

    def check_input_types(self, input_):
        """
        Checks if input to functions' types matches previous inputs
        """
        assert guard(self.input_types, list)
        self._check_types(self.input_types, input_)

    def check_output_types(self, output):
        """
        Checks if outputs to functions' types matches previous outputs
        """
        assert guard(self.output_types, list)
        self._check_types(self.output_types, output)

    def create_generator_instance(self, name: str) -> Instance:
        """
        Create generator instance
        """
        self.validate()

        inst_input_vars: list[Var] = list(
            map(
                lambda var: ExclusiveVar(f"{name}_{self.name}_{var.py_name}"),
                self.input_vars,
            )
        )
        inst_output_vars: list[Var] = list(
            map(
                lambda var: ExclusiveVar(f"{name}_{self.name}_{var.py_name}"),
                self.output_vars,
            )
        )

        signals = ProtocolSignals(prefix=f"{self.prefix}{self.name}_{name}__")

        return Instance(
            self.name,
            Var(name),
            inst_input_vars,
            inst_output_vars,
            signals,
        )

    def make_var(self, name: str) -> Var:
        """
        Makes a variable with own prefix
        """
        return Var(py_name=f"{self.prefix}{name}", ver_name=f"_{self.prefix}{name}")
