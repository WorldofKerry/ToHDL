"""
Intermediate Representation Expressions

Represents the subset of Python expressions
that are synthesizable

"""
from __future__ import annotations

import copy

from python2verilog.utils.assertions import assert_typed, get_typed, get_typed_list
from python2verilog.utils.generics import GenericRepr


class Expression(GenericRepr):
    """
    An expression that can be equated
    """

    def __init__(self, string: str):
        assert isinstance(string, str)
        if "_state" in string and self.__class__ == Expression:
            raise RuntimeError()
        self.string = string

    def to_string(self) -> str:
        """
        To String
        """
        return self.string

    def __str__(self):
        return self.to_string()

    def __eq__(self, other: object):
        if isinstance(other, Expression):
            return self.verilog() == other.verilog()
        return False

    def __hash__(self):
        return hash(self.to_string())

    def verilog(self) -> str:
        """
        In Verilog syntax
        """
        return self.to_string()


class Int(Expression):
    """
    Signed integer literal
    """

    def __init__(self, value: int):
        self.value = get_typed(value, int)
        super().__init__(str(self.__class__))

    def verilog(self) -> str:
        """
        In Verilog
        """
        return f"$signed({str(self.value)})"

    def to_string(self) -> str:
        """
        String
        """
        return str(self.value)


class UInt(Expression):
    """
    Unsigned integer literal
    """

    def __init__(self, value: int):
        assert isinstance(value, int)
        super().__init__(str(value))


class Unknown(Expression):
    """
    Unknown or "don't care" value
    """

    def __init__(self):
        super().__init__("'x")


class Var(Expression):
    """
    Named-variable
    """

    def __init__(
        self,
        py_name: str,
        ver_name: str = "",
        width: int = 32,
        is_signed: bool = True,
        initial_value: str = "0",
        **_,
    ):
        if ver_name == "":
            ver_name = "_" + py_name

        self.ver_name = get_typed(ver_name, str)
        self.py_name = get_typed(py_name, str)  # Matches I/O of Verilog
        self.width = get_typed(width, int)
        self.is_signed = get_typed(is_signed, bool)
        self.initial_value = initial_value

        super().__init__(ver_name)


class State(Var):
    """
    State constant
    """

    def __init__(
        self, name, width: int = 32, isSigned: bool = True, initial_value: str = "0"
    ):
        super().__init__(
            name, name, width=width, is_signed=isSigned, initial_value=initial_value
        )


class Ternary(Expression):
    """
    <condition> ? <left> : <right>
    """

    def __init__(self, condition: Expression, left: Expression, right: Expression):
        self.condition = condition
        self.left = left
        self.right = right
        super().__init__(self.to_string())

    def to_string(self):
        return (
            f"({self.condition.to_string()} ? {self.left.to_string()}"
            f" : {self.right.to_string()})"
        )

    def verilog(self):
        return f"({self.condition.verilog()} ? {self.left.verilog()} : {self.right.verilog()})"


class UBinOp(Expression):
    """
    Unsigned BinOp
    Is usually better for comparators
    """

    def __init__(self, left: Expression, oper: str, right: Expression):
        self._left = get_typed(left, Expression)
        self._right = get_typed(right, Expression)
        self._oper = get_typed(oper, str)
        super().__init__(self.__class__.__name__)

    @property
    def left(self):
        """
        lvalue
        """
        return copy.deepcopy(self._left)

    @left.setter
    def left(self, other: Expression):
        self._left = get_typed(other, Expression)

    @property
    def right(self):
        """
        rvalue
        """
        return copy.deepcopy(self._right)

    @right.setter
    def right(self, other: Expression):
        self._right = get_typed(other, Expression)

    def to_string(self):
        return f"({self._left.to_string()} {self._oper} {self._right.to_string()})"

    def verilog(self):
        """
        To Verilog
        """
        return f"({self._left.verilog()} {self._oper} {self._right.verilog()})"


class BinOp(UBinOp):
    """
    <left> <op> <right>

    In verilog the signed specifier is used.
    For mixed unsigned and signed operations, the following page explains well
    https://www.01signal.com/verilog-design/arithmetic/signed-wire-reg/
    """

    def verilog(self):
        return "$signed" + super().verilog()


class Add(BinOp):
    """
    <left> + <right>
    """

    def __init__(self, left: Expression, right: Expression):
        super().__init__(left, "+", right)


class Sub(BinOp):
    """
    <left> - <right>
    """

    def __init__(self, left: Expression, right: Expression):
        super().__init__(left, "-", right)


class Mul(BinOp):
    """
    <left> * <right>
    """

    def __init__(self, left: Expression, right: Expression):
        super().__init__(left, "*", right)


class Div(BinOp):
    """
    <left> / <right>
    """

    def __init__(self, left: Expression, right: Expression):
        super().__init__(left, "/", right)


class LessThan(UBinOp):
    """
    <left> < <right>
    """

    def __init__(self, left: Expression, right: Expression):
        super().__init__(left, "<", right)


class Pow(UBinOp):
    """
    <left> ** <right>
    """

    def __init__(self, left: Expression, right: Expression):
        super().__init__(left, "**", right)


class _Mod(UBinOp):
    """
    <left> % <right>
    """

    def __init__(self, left: Expression, right: Expression):
        super().__init__(left, "%", right)


class UnaryOp(Expression):
    """
    <op>(<expr>)
    """

    def __init__(self, oper: str, expr: Expression):
        self.oper = get_typed(oper, str)
        self.expr = get_typed(expr, Expression)
        super().__init__(self.__class__.__name__)

    def to_string(self):
        """
        string
        """
        return f"{self.oper}({self.expr.to_string()})"

    def verilog(self):
        """
        Verilog
        """
        return f"{self.oper}({self.expr.verilog()})"


class Mod(BinOp):
    """
    <left> % <right>
    """

    def __init__(self, left: Expression, right: Expression):
        self.left = get_typed(left, Expression)
        self.right = get_typed(right, Expression)
        super().__init__(left, "%", right)

    def verilog(self):
        """
        Verilog
        """
        return Ternary(
            UBinOp(self.left, "<", Int(0)),
            Ternary(
                UBinOp(self.right, ">=", Int(0)),
                UnaryOp("-", _Mod(self.left, self.right)),
                _Mod(self.left, self.right),
            ),
            Ternary(
                UBinOp(self.right, "<", Int(0)),
                UnaryOp("-", _Mod(self.left, self.right)),
                _Mod(self.left, self.right),
            ),
        ).verilog()

    def to_string(self):
        """
        String
        """
        return f"({self.left.to_string()} % {self.right.to_string()})"


class FloorDiv(BinOp):
    """
    <left> // <right>

    Follows Python conventions
    """

    def __init__(self, left: Expression, right: Expression):
        self.left = get_typed(left, Expression)
        self.right = get_typed(right, Expression)
        super().__init__(left, "//", right)

    def verilog(self):
        """
        Verilog
        """
        return Ternary(
            condition=BinOp(
                left=BinOp(left=self.left, right=self.right, oper="%"),
                right=Int(0),
                oper="===",
            ),
            left=BinOp(self.left, "/", self.right),
            right=BinOp(
                BinOp(self.left, "/", self.right),
                "-",
                BinOp(
                    UBinOp(
                        BinOp(self.left, "<", Int(0)),
                        "^",
                        BinOp(self.right, "<", Int(0)),
                    ),
                    "&",
                    Int(1),
                ),
            ),
        ).verilog()
