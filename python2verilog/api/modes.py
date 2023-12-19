"""
Modes
"""

from __future__ import annotations

from enum import Enum


class Modes(Enum):
    """
    Filesystem Modes
    """

    NO_WRITE = 0
    WRITE = 1
    OVERWRITE = 2

    @staticmethod
    def write(mode: Modes) -> bool:
        """
        Returns if user wants to write
        """
        return mode in (Modes.WRITE, Modes.OVERWRITE)

    @staticmethod
    def open_text_mode(mode: Modes) -> str:
        """
        Returns proper mode argument for `open`
        """
        assert Modes.write(mode)
        return "w" if mode == Modes.OVERWRITE else "x"
