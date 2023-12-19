"""
Special env variable functions
"""

import os
from enum import Enum
from typing import Optional

PREFIX = "PYTHON_2_VERILOG_"


class Vars(Enum):
    """
    Env var names
    """

    # Set to turn on compute-heavy assertions
    DEBUG_MODE = PREFIX + "DEBUG"

    # Path to iverilog
    IVERILOG_PATH = PREFIX + "IVERILOG_PATH"

    # Set to enable new SystemVerilog features
    IS_SYSTEM_VERILOG = PREFIX + "SYSTEM_VERILOG"

    # Set if no write to filesystem at system exit
    NO_WRITE_TO_FS = PREFIX + "NO_WRITE_TO_FS"

    # Add debug comments
    DEBUG_COMMENTS = PREFIX + "DEBUG_COMMENTS"


def set_debug_mode(mode: bool):
    """
    Sets the debug mode of package
    """
    if mode:
        set_var(Vars.DEBUG_MODE, "")
    else:
        set_var(Vars.DEBUG_MODE, None)


def is_debug_mode():
    """
    :return: True if in debug mode, false otherwise
    """
    return get_var(Vars.DEBUG_MODE) is not None


def set_var(name: Vars, value: Optional[str]):
    """
    Set an env var
    """
    assert isinstance(name, Vars)
    assert isinstance(value, (type(None), str))
    if value is not None:
        os.environ[str(name)] = value
    else:
        try:
            del os.environ[str(name)]
        except KeyError:
            pass


def get_var(name: Vars):
    """
    Get an env var
    """
    assert isinstance(name, Vars)
    return os.getenv(str(name))
