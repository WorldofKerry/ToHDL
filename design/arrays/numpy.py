code = """
import numpy as np
val = np.ndarray(10)

def func(input: np.ndarray) -> tuple[np.ndarray]:
  return np.ndarray(15)
"""

import ast

tree = ast.parse(code)
print(ast.dump(tree, indent=4))
