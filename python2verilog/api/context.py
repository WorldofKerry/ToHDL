"""
Functions that take text as input
"""

import logging
import textwrap

import pytohdl  # pylint: disable=import-error

from python2verilog import ir
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
    ver_code_gen, _ = context_to_codegen(context)

    # if context.is_generator:
    # try:
    generators = []
    for v in context.namespace.values():
        if v.is_generator:
            generators.append(v.name)
    assert (
        len(generators) <= 1
    ), f"Only one generator function allowed in namespace {generators}"

    functions = {
        k: textwrap.dedent(v.py_string or "") for k, v in context.namespace.items()
    }
    module_str = pytohdl.translate(  # pylint: disable=no-member
        pytohdl.PyContext(context.name, functions)  # pylint: disable=no-member
    )
    # logging.error("Path 1")
    # except BaseException as e:  # pylint: disable=broad-exception-caught
    #     module_str = ver_code_gen.get_module_str()
    #     logging.info(
    #         "Failed to use Rust backend, falling back to Python backend with error: %s",
    #         e,
    #     )
    # else:
    #     module_str = ver_code_gen.get_module_str()
    #     # logging.warning("Path 3")

    tb_str = ver_code_gen.get_testbench_str(config)
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
