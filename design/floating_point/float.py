from __future__ import annotations
from dataclasses import dataclass
import copy
import struct
import textwrap


def ppbin(b: int):
    """
    Pretty-prints as binary
    """
    b = bin(b)[2:]  # remove leading `0b`
    return "_".join(textwrap.wrap(b[::-1], 4))[::-1]


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

    def __lt__(self, other: Float) -> bool:
        if self.sign != other.sign:
            return self.sign
        if self.exponent != other.exponent:
            return self.exponent < other.exponent
        return self.mantissa < other.mantissa

    def __add__(self, other: Float) -> Float:
        """
        Based on https://www.doc.ic.ac.uk/~eedwards/compsys/float/
        """
        a = copy.deepcopy(self)
        b = copy.deepcopy(other)
        c = Float.zero()

        # Make sure a has larger by magnitude
        if a.exponent < b.exponent:
            a, b = b, a
        elif a.exponent == b.exponent:
            if a.mantissa < b.mantissa:
                a, b = b, a

        c.sign = a.sign

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

        subtract = a.sign ^ b.sign
        print(f"{subtract=}")

        if subtract:
            c.mantissa = a.mantissa - b.mantissa
        else:
            c.mantissa = a.mantissa + b.mantissa

        c.exponent = a.exponent
        print(f"{ppbin(c.mantissa)=}")

        # Normalize
        msb_index = 0
        temp = c.mantissa
        logial_shift_right = lambda val, n: (
            val >> n if val >= 0 else (val + 2**24) >> n
        )
        while temp:
            print(f"stuck {temp=} {msb_index}")
            temp = logial_shift_right(temp, 1)
            msb_index += 1
        print(f"{msb_index=}")

        # Shift left until implicit bit is MSB
        # Decrease exponent to match
        left_shift_amount = 24 - msb_index  # Can be negative
        if left_shift_amount >= 0:
            c.mantissa <<= left_shift_amount
            c.exponent -= left_shift_amount
        else:
            c.mantissa >>= -left_shift_amount
            c.exponent += -left_shift_amount

        c.mantissa &= 2**23 - 1

        print(f"{ppbin(c.mantissa)=}")

        return c


def test_sum_mixed():

    def inner(a, b):
        f1 = Float.from_float(a)
        f2 = Float.from_float(b)
        sum = f1 + f2

        print(f"{sum=}")
        print(f"{sum.as_decimal()=}")
        assert sum.as_decimal() == a + b

    inner(0.5, -0.4375)

    # Same exponent
    inner(123, 124)
    inner(124, 123)
    inner(-124, 123)
    inner(124, -123)
    inner(-124, -123)


def test_comparision():
    zero = Float.from_float(0.0)
    plus_one = Float.from_float(1.0)
    minus_one = Float.from_float(-1.0)
    assert zero < plus_one
    assert minus_one < zero
    assert minus_one < plus_one
    assert Float.from_float(-124) < Float.from_float(123)


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
    f1 = Float.from_float(4.0)
    f2 = Float.from_float(1.0)
    sum = f1 + f2

    print(f"{sum=}")
    print(f"{sum.as_decimal()=}")
    assert sum.as_decimal() == 5


def main():
    test_representation()
    test_sum_positives()
    test_comparision()
    test_sum_mixed()
    return


if __name__ == "__main__":
    main()
