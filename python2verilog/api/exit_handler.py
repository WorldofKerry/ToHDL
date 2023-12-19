"""
Handles namespaces
"""

from __future__ import annotations

from typing import Any, Callable

from python2verilog.api.file_namespaces import _file_namespaces
from python2verilog.api.namespace import namespace_to_file
from python2verilog.utils import env

try:
    # For iPython
    def __exit_register(fun: Callable[[], Any], *_args, **_kwargs):
        """Decorator that registers at post_execute. After its execution it
        unregisters itself for subsequent runs."""

        def callback():
            fun()
            ip.events.unregister("post_execute", callback)

        ip.events.register("post_execute", callback)

    ip = get_ipython()  # type: ignore

except NameError:
    # For normal
    from atexit import register as __exit_register  # type: ignore


@__exit_register
def __namespace_exit_handler():
    """
    Handles the conversions in each namespace for program exit
    """
    if env.get_var(env.Vars.NO_WRITE_TO_FS) is None:
        namespace_exit_handler()


def namespace_exit_handler():
    """
    Handles the conversions in each namespace for program exit
    """
    for stem, namespace in _file_namespaces.items():
        namespace_to_file(stem, namespace)
