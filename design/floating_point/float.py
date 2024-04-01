from __future__ import annotations
from dataclasses import dataclass
import copy
import struct


@dataclass
class FloatingPointFormat:
    mantissa_bits: int
    exponent_bits: int


class Float:
    def __init__(self, sign, exponent, mantissa) -> None:
        self.sign = sign
        self.exponent = exponent
        self.mantissa = mantissa

    @classmethod
    def from_hex(cls, hex):
        mantissa = (hex >> 0) & (2**23 - 1)
        exponent = (hex >> 23) & (2**8 - 1)
        sign = (hex >> 31) & (2**1 - 1)
        return cls(sign=sign, exponent=exponent, mantissa=mantissa)

    @classmethod
    def from_float(cls, f):
        # Based on https://stackoverflow.com/a/23624284
        h = hex(struct.unpack("<I", struct.pack("<f", f))[0])
        return Float.from_hex(int(h, 0))

    @classmethod
    def zero(cls):
        return cls(sign=0, exponent=0, mantissa=0)

    def as_decimal(self) -> int:
        mantissa = 1  # hidden 1
        for up, down in enumerate(reversed(range(24))):  # 23 mantissa bits
            bit = (self.mantissa >> down) & 1
            if bit:  # assume normal
                mantissa += 1 / (2 ** (up))

        exponent = self.exponent - 127

        decimal = (-1) ** self.sign * mantissa * 2**exponent

        return decimal

    def __repr__(self) -> str:
        return f"{Float.__name__}({self.sign=},{self.exponent=},{self.mantissa=})"

    def __add__(self, other: Float) -> Float:
        a = copy.deepcopy(self)
        b = copy.deepcopy(other)
        c = Float.zero()

        # a is larger
        if a.as_decimal() < b.as_decimal():
            a, b = b, a

        print(f"{a.as_decimal()=}")
        print(f"{b.as_decimal()=}")

        print(f"{a=}")
        print(f"{b=}")

        # Add implicit one
        a.mantissa |= 1 << 23
        b.mantissa |= 1 << 23

        # Adjust the smaller mantissa so exponents are same
        exponent_difference = a.exponent - b.exponent
        print(f"{exponent_difference=}")
        b.mantissa >>= exponent_difference

        c.mantissa = a.mantissa + b.mantissa
        print(f"{a.mantissa}")
        print(f"{b.mantissa}")
        print(f"{c.mantissa=}")
        print(f"{bin(c.mantissa)=}")
        c.mantissa = c.mantissa & (2**23 - 1)  # remove implicit one
        print(f"{bin(c.mantissa)=}")
        c.exponent = a.exponent

        return c


def test_representation():
    f1 = Float.from_hex(0xC3064000)  # -134.25
    f2 = Float.from_hex(0x4300A000)  # 128.625
    print(f"{f1=}")
    print(f"{f1.as_decimal()=}")
    assert f1.as_decimal() == -134.25
    print(f"{f2=}")
    print(f"{f2.as_decimal()=}")
    assert f2.as_decimal() == 128.625


def test_sum_positives():
    f1 = Float.from_float(2.0)
    f2 = Float.from_float(1.0)
    sum = f1 + f2

    print(f"{sum=}")
    print(f"{sum.as_decimal()=}")
    assert sum.as_decimal() == 3

def test_sum_mixed():
    f1 = Float.from_float(2.0)
    f2 = Float.from_float(1.0)
    sum = f1 + f2

    print(f"{sum=}")
    print(f"{sum.as_decimal()=}")
    assert sum.as_decimal() == 3

def main():
    test_representation()
    test_sum_positives()
    # test_sum_mixed()
    return


if __name__ == "__main__":
    main()
