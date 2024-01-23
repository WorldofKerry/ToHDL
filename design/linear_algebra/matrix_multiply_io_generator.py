"""
A routine for matrix multiply that uses yield and send to query / store specific elements in the matrix,

taking up O(1) space including i/o. 

"""
from typing import Generator
import numpy as np


def matrix_multiply(
    m: int, n: int, p: int
) -> Generator[tuple[int, int, int, int, int], int, None]:
    """
    Matrix multiply C = AB
    where A: m x n, B: m x p, C: n x p

    Requests values and expects values
    """
    for i in range(n):
        for j in range(p):
            sum = 0
            for k in range(m):
                # breakpoint()
                yield 0, None, 0, i, k
                a_ik = yield 0, None, 1, k, j
                int(a_ik)
                b_kj = yield 0, None, 0, None, None
                int(b_kj), b_kj
                sum = sum + a_ik * b_kj
            print("??", type(sum))
            yield 0, sum, 2, i, j
        yield 1, None, None, None, None


def main():
    """
    a * b = c
    """
    a = np.random.randint(1, 5, size=(2, 2))
    b = np.random.randint(1, 5, size=(2, 2))
    c = np.zeros_like(a)

    print(a)
    print(b)
    print(a @ b)
    print(int(a[0][0]))

    mm = matrix_multiply(2, 2, 2)
    done, rw, sel, x, y = next(mm)
    while not done:
        match sel:
            case 0:
                temp = a[x][y]
                try:
                    int(temp)
                except:
                    print(locals())
                    int(temp)
                done, rw, sel, x, y = mm.send(temp)
            case 1:
                temp = b[x][y]
                int(temp)
                done, rw, sel, x, y = mm.send(temp)
            case 2:
                print(f"{done=} {rw=} {sel=} {x=} {y=}")
                print(type(rw))
                c[x, y] = rw
                done, rw, sel, x, y = mm.send(None)
        print(f"loop {x=} {y=}")

    print(c)


if __name__ == "__main__":
    main()
