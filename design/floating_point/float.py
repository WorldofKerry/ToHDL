from __future__ import annotations


class Float:
    def __init__(self, hex) -> None:
        self.mantissa = (hex >> 0) & (2**23 - 1)
        self.exponent = (hex >> 23) & (2**8 - 1)
        self.sign = (hex >> 31) & (2**1 - 1)

    def str_decimal(self) -> str:
        mantissa = 1  # hidden 1
        for up, down in enumerate(reversed(range(23))):  # 23 mantissa bits
            bit = (self.mantissa >> down) & 1
            if bit:  # assume normal
                mantissa += 1 / (2 ** (up + 1))

        exponent = self.exponent - 127

        decimal = -1 if self.sign else 1
        decimal *= mantissa * 2**exponent

        return f"{mantissa=} {exponent=} {decimal=}"

    def __repr__(self) -> str:
        return f"{Float.__name__}({self.sign=},{self.exponent=},{self.mantissa=})"

    def __add__(self, other) -> Float:
        return Float(0)


def main():
    f1 = Float(0xC3064000)  # -134.25
    f2 = Float(0x4300A000)  # 128.625
    print(f"{f1=}")
    print(f"{f1.str_decimal()=}")
    print(f"{f2=}")
    print(f"{f2.str_decimal()=}")

    sum = f1 + f2  #

    print(f"{sum=}")
    return


if __name__ == "__main__":
    main()
