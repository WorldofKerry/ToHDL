"""
Functions that take text as input
"""

import inspect
import logging
import textwrap

from python2verilog import ir, pytohdl
from python2verilog.backend import verilog
from python2verilog.backend.verilog.config import CodegenConfig, TestbenchConfig
from python2verilog.frontend.function import Function
from python2verilog.optimizer import IncreaseWorkPerClockCycle
from python2verilog.utils.typed import typed


def context_to_codegen(context: ir.Context):
    """
    Converts a context to verilog and its ir

    :return: (codegen, ir)
    """
    context.validate()
    context, ir_root = Function(context).parse_function()
    context.freeze()
    logging.debug(
        "context to codegen %s %s -O%s with %s",
        ir_root.unique_id,
        context.name,
        context.optimization_level,
        context,
    )
    if context.optimization_level > 0:
        logging.info("Running %s", IncreaseWorkPerClockCycle.__name__)
        IncreaseWorkPerClockCycle(ir_root, threshold=context.optimization_level - 1)
    logging.info("Running %s", verilog.CodeGen.__name__)
    return verilog.CodeGen(ir_root, context), ir_root


def context_to_verilog(context: ir.Context, config: CodegenConfig) -> tuple[str, str]:
    """
    Converts a context to a verilog module and testbench

    :return: (module, testbench)
    """
    typed(context, ir.Context)
    # ver_code_gen, _ = context_to_codegen(context)
    logging.debug("context_to_verilog")

    try:
        # assert (
        #     context.optimization_level == 0
        # ), f"No real optimization exists for Rust backend {context.optimization_level}"
        generators = []
        for v in context.namespace.values():
            if v.is_generator:
                generators.append(v.name)
        assert (
            len(generators) <= 1
        ), f"Only one generator function allowed in namespace {generators}"
        assert len(context.namespace) <= 4, "Only small namespaces allowed"
        functions = {
            k: textwrap.dedent(v.py_string or "") for k, v in context.namespace.items()
        }
        print(f"{context.namespace.keys()=}")
        functions_new = dict()
        for k, v in context.namespace.items():
            functions_new[k] = textwrap.dedent(v.py_string or "")
            for g in v.py_func.__globals__:
                if not g.startswith("_"):
                    functions_new[g] = textwrap.dedent(
                        inspect.getsource(v.py_func.__globals__[g])
                    )
        print(f"{context.name=}\n{functions_new.keys()=}\n{functions.keys()=}")
        module_str = pytohdl.translate(  # pylint: disable=no-member
            pytohdl.PyContext(context.name, functions_new)  # pylint: disable=no-member
        )
    # except AssertionError:
    #     module_str = ver_code_gen.get_module_str()
    # except BaseException as e:  # pylint: disable=broad-exception-caught
    #     assert "pyo3_runtime" in str(e.__class__), str(e)
    #     module_str = ver_code_gen.get_module_str()
    #     logging.info(
    #         "Failed to use Rust backend, falling back to Python backend with error: %s",
    #         e,
    #     )
    finally:
        pass

    tb_str = ""
    return module_str, tb_str


def context_to_verilog_and_dump(context: ir.Context) -> tuple[str, str, str]:
    """
    Converts a context to a verilog module, testbench, and cytoscape str

    :return: (module, testbench, cytoscape_dump) pair
    """
    typed(context, ir.Context)
    ver_code_gen, ir_root = context_to_codegen(context)

    module_str = ver_code_gen.get_module_str()
    tb_str = ver_code_gen.get_testbench_str(TestbenchConfig())

    return module_str, tb_str, ir.create_cytoscape_elements(ir_root)
