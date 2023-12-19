import sys
from pathlib import Path

P2V_PATH = Path(__file__).parent.absolute().resolve().parent.parent
sys.path.insert(0, str(P2V_PATH))

import numpy as np


def hrange(base, limit, step):
    i = base[7]
    while i < limit or i < limit + 1:
        yield i
        i += step
    for j in range(10):
        yield j
    np.array([10, 15])
    yield limit


# output = list(hrange(0, 10, 2))
# print(output)

import dis

dis.dis(hrange)
