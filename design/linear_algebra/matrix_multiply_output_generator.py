"""
A routine for matrix multiply that yields elements of the output.

Takes O(mn + mp) space including i/o.

"""
from typing import Generator, Iterator
import numpy as np


def matrix_multiply(a, b, m: int, n: int, p: int) -> Iterator[tuple[int, int, int]]:
    """
    Matrix multiply C = AB
    where A: m x n, B: m x p, C: n x p

    Requests values and expects values
    """
    for i in range(n):
        for j in range(p):
            sum = 0
            for k in range(m):
                a_ik = a[i, k]
                b_kj = b[k, j]
                sum = sum + a_ik * b_kj
            yield sum


def main():
    """
    a * b = c
    """
    m = 10
    n = 10
    p = 10

    a = np.random.randint(-100, 100, size=(m, n))
    b = np.random.randint(-100, 100, size=(m, p))
    c = np.zeros(n * p, dtype=np.int32)

    print(a)
    print(b)
    print(a @ b)

    mm = matrix_multiply(a, b, m, n, p)
    for i, sum in enumerate(mm):
        c[i] = sum

    c = c.reshape(n, p)
    print(c)

    assert np.array_equal(c, a @ b)


if __name__ == "__main__":
    main()
