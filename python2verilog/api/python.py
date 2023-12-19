"""
Functions that take text as input
"""

import ast
import logging
from types import FunctionType
from typing import Any, Optional

from python2verilog import ir
from python2verilog.api.context import context_to_codegen
from python2verilog.api.modes import Modes
from python2verilog.api.namespace import get_namespace
from python2verilog.backend.verilog.config import CodegenConfig, TestbenchConfig
from python2verilog.utils.typed import guard, typed, typed_list


def py_to_codegen(
    code: str,
    function_name: str,
    write: bool,
    extra_test_cases: Optional[list[tuple[int, ...]]] = None,
    file_path: str = "",
    optimization_level: int = 1,
):
    """
    Converts from code to verilog code generator

    :return: (context, ir)
    """
    context = py_to_context(
        code=code,
        function_name=function_name,
        extra_test_cases=extra_test_cases,
        file_path=file_path,
        write=write,
        optimization_level=optimization_level,
    )
    assert isinstance(context, ir.Context)
    assert isinstance(extra_test_cases, list)
    context.optimization_level = optimization_level
    context.test_cases = extra_test_cases
    return context_to_codegen(context)


def py_to_verilog(
    code: str,
    function_name: str,
    extra_test_cases: Optional[list[tuple[int, ...]]] = None,
    file_path: str = "",
    write: bool = True,
    optimization_level: int = 1,
):
    """
    Converts from code to module and testbench strings

    :return: (module, testbench)
    """
    typed(code, str)
    typed(function_name, str)
    assert function_name in code
    typed_list(extra_test_cases, tuple)  # type: ignore[misc]
    typed(file_path, str)

    code_gen, _ = py_to_codegen(
        code=code,
        function_name=function_name,
        extra_test_cases=extra_test_cases,
        file_path=file_path,
        optimization_level=optimization_level,
        write=write,
    )
    return code_gen.get_module_str(), code_gen.get_testbench_str(TestbenchConfig())


def py_to_context(
    code: str,
    function_name: str,
    file_path: str,
    write: bool,
    optimization_level: int,
    extra_test_cases: Optional[list[tuple[int, ...]]] = None,
):
    """
    Parses python code into the function and testbench

    :return: context
    """
    # pylint: disable=too-many-locals
    typed(code, str)
    typed(function_name, str)

    def get_file_and_line_num(node: ast.AST):
        """
        Gets file and line number
        """
        if file_path:
            string = str(file_path)
        else:
            string = "line"
        string += f":{node.lineno}"
        return string

    tree = ast.parse(code)

    test_cases = extra_test_cases if extra_test_cases else []

    for node in ast.walk(tree):
        # logging.debug(f"Walking through {ast.dump(node)}")
        if isinstance(node, ast.FunctionDef) and node.name == function_name:
            logging.info("Found function at %s", get_file_and_line_num(node))
            generator_ast = node
        elif (
            isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name)
            and node.func.id == function_name
        ):
            func_call_str = ast.get_source_segment(code, node)
            assert func_call_str
            func_call_str = "(" + func_call_str.split("(", 1)[1]
            func_call_str = func_call_str.rsplit(")", 1)[0] + ")"

            try:
                output = ast.literal_eval(func_call_str)
                if not isinstance(output, tuple):
                    output = (output,)
                test_cases.append(output)

                logging.info(
                    "Found test case at %s with %s", get_file_and_line_num(node), output
                )
            except ValueError as e:
                raise ValueError(
                    f"Attempted to literally eval {func_call_str} at "
                    f"{file_path}:{node.lineno}, but got non-literals"
                ) from e

    logging.info("Test cases: %s", test_cases)

    try:
        input_names = [var.arg for var in generator_ast.args.args]
    except UnboundLocalError as e:
        raise RuntimeError(f"Didn't find `{function_name}` in file") from e

    logging.info("Input param names %s", input_names)

    initialized = False
    input_types: Optional[list[type[Any]]] = None
    for output in test_cases:
        if not initialized:
            input_types = [type(val) for val in output]
            initialized = True
        assert guard(input_types, list)
        for i, (expected_type, actual_value) in enumerate(zip(input_types, output)):
            assert expected_type == type(
                actual_value
            ), f"Expected parameter `{input_names[i]}` to be \
                {expected_type} but got {type(actual_value)} instead"

    logging.info("Input param types %s", input_types)

    locals_: dict[str, FunctionType] = {}
    lines = code.splitlines()
    func_lines = lines[generator_ast.lineno - 1 : generator_ast.end_lineno]
    func_str = "\n".join(func_lines)
    exec(func_str, None, locals_)
    try:
        generator_func = locals_[function_name]
    except KeyError as e:
        raise RuntimeError(f"def {function_name} not found in global context") from e

    initialized = False

    for output in test_cases:
        generator = generator_func(*output)

        if not initialized:
            result = next(generator)
            if not isinstance(result, tuple):
                result = (result,)
            output_types = [type(val) for val in result]
            initialized = True

        for iter_, output in enumerate(generator):
            if not isinstance(output, tuple):
                output = (output,)

            for i, (expected_type, actual_value) in enumerate(
                zip(output_types, output)
            ):
                assert expected_type == type(
                    actual_value
                ), f"Expected parameter `{input_names[i]}` to be \
                    {expected_type} but got {type(actual_value)} instead"

            if iter_ >= 100000:
                err_msg = f"capped generator outputs to {iter_} iterations"
                logging.error(err_msg)
                raise RuntimeError(err_msg)

    context = ir.Context(name=function_name)

    if initialized:
        context.output_types = output_types
        context.default_output_vars()

        logging.info("Output param types %s", context.output_types)
        logging.info("Output param names %s", context.output_vars)

    context.input_vars = [ir.Var(name) for name in input_names]
    if input_types is not None:
        assert isinstance(input_types, list)
        context.input_types = input_types
    context.test_cases = test_cases
    context.py_ast = generator_ast
    context.py_func = generator_func
    context.py_string = func_str
    context.optimization_level = optimization_level
    context.mode = Modes.OVERWRITE if write else Modes.NO_WRITE
    context.namespace = get_namespace(file_path)
    context.namespace[function_name] = context

    return context
