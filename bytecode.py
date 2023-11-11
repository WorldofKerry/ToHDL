import dis
def func(n):
   a, b, (c, d) = 1, 2, 10, 11, (n, 4)
dis.dis(func)