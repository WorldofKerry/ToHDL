"""
Python to Verilog translater based on ToHDL
"""

class PyContext:
    """
    Context for a Python function and all of its called upon functions
    """

    def __init__(self, main: str, functions: dict[str, str]): ...

def translate(context: PyContext) -> str: ...
