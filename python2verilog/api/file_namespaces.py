"""
Handles namespaces
"""

from __future__ import annotations

from pathlib import Path

from python2verilog import ir

_file_namespaces: dict[Path, dict[str, ir.Context]] = {}
