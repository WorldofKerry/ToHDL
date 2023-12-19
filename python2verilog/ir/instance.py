"""
An instance of a generator

Can be created from a context
"""

from dataclasses import dataclass

from python2verilog.ir.expressions import Var
from python2verilog.ir.signals import ProtocolSignals


@dataclass(frozen=True)
class Instance:
    """
    Instance

    :param module_name: generator function name
    :param var: variable name assigned to generator instance
    """

    # pylint: disable=too-many-instance-attributes
    module_name: str
    var: Var
    inputs: list[Var]
    outputs: list[Var]
    signals: ProtocolSignals
