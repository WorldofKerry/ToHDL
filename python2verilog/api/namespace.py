"""
Handles namespaces
"""

from __future__ import annotations

from pathlib import Path
from typing import Optional

from python2verilog import ir
from python2verilog.api.context import context_to_verilog
from python2verilog.api.file_namespaces import _file_namespaces
from python2verilog.api.modes import Modes
from python2verilog.backend.verilog.config import CodegenConfig


def get_namespace(path: Path | str) -> dict[str, ir.Context]:
    """
    Get namespace of a file and creates it if it doesn't exist

    Only the path without extension is used, e.g.,

    - `/path/to/file` -> `/path/to/file` namespace
    - `/path/to/file.py` -> `/path/to/file` same namespace as above
    - `/path/to/file.ext` -> `/path/to/file` same namespace as above
    """
    path = Path(path)
    namespace = path.with_suffix("")
    if namespace not in _file_namespaces:
        _file_namespaces[namespace] = {}
    return _file_namespaces[namespace]


def new_namespace(path: Path | str) -> dict[str, ir.Context]:
    """
    Create a new namespace for path

    Only the path without extension is used, e.g.,

    - `/path/to/file` -> `/path/to/file` namespace
    - `/path/to/file.py` -> `/path/to/file` same namespace as above
    - `/path/to/file.ext` -> `/path/to/file` same namespace as above
    """
    path = Path(path)
    namespace = path.with_suffix("")
    assert (
        namespace not in _file_namespaces
    ), f"Namespace for {namespace} already exists"
    return get_namespace(namespace)


def namespace_to_file(
    path: Path, namespace: dict[str, ir.Context], config: Optional[CodegenConfig] = None
) -> tuple[str, str]:
    """
    Writes modules and testbenches files

    :return: (modules, testbenches) for convenience
    """
    if not config:
        config = CodegenConfig()

    module, testbench = namespace_to_verilog(namespace, config)

    if all(map(lambda ns: ns.mode == Modes.OVERWRITE, namespace.values())):
        mode = "w"
    elif all(map(lambda ns: Modes.write(ns.mode), namespace.values())):
        mode = "x"
    else:
        return module, testbench

    with open(str(path) + ".sv", mode=mode, encoding="utf8") as module_file, open(
        str(path) + "_tb.sv", mode=mode, encoding="utf8"
    ) as testbench_file:
        module_file.write(module)
        testbench_file.write(testbench)
    return module, testbench


def namespace_to_verilog(
    namespace: dict[str, ir.Context], config: Optional[CodegenConfig] = None
) -> tuple[str, str]:
    """
    Namespace to modules and testbenches str

    :return: (modules, testbenches)
    """
    if not config:
        config = CodegenConfig()
    module = []
    testbench = []
    for context in namespace.values():
        mod, tb = context_to_verilog(context=context, config=config)
        module.append(mod)
        testbench.append(tb)
    return "".join(module), "".join(testbench)
