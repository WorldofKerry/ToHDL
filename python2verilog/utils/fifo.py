"""
Fifo wrappers
"""

import os
import tempfile
from contextlib import contextmanager
from typing import Iterator


@contextmanager
def temp_fifo() -> Iterator[str]:
    """
    Create a temporary fifo
    """
    tmpdir = tempfile.mkdtemp()
    filename = os.path.join(tmpdir, "fifo")  # Temporary filename
    os.mkfifo(filename)  # Create FIFO
    try:
        yield filename
    finally:
        os.unlink(filename)  # Remove file
        os.rmdir(tmpdir)  # Remove directory
