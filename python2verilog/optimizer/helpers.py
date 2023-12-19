"""
Optimizer helper functions
"""

import copy

from python2verilog import ir


def backwards_replace(
    expr: ir.Expression, mapping: dict[ir.Var, ir.Expression]
) -> ir.Expression:
    """
    If the expression matches a key in the mapping, it is replaced with
    the corresponding value in the mapping.

    Note: ignores exclusive vars in replacement process

    :return: a copy of the updated expression.
    """
    expr = copy.deepcopy(expr)
    if isinstance(expr, ir.Var):
        for key in mapping:
            if key.to_string() == expr.to_string():
                return mapping[key]
    elif isinstance(expr, (ir.UInt, ir.Int)):
        return expr
    elif isinstance(expr, (ir.BinOp, ir.UBinOp)):
        expr.left = backwards_replace(expr.left, mapping)
        expr.right = backwards_replace(expr.right, mapping)
    elif isinstance(expr, ir.Ternary):
        expr.condition = backwards_replace(expr.condition, mapping)
        expr.left = backwards_replace(expr.left, mapping)
        expr.right = backwards_replace(expr.right, mapping)
    elif isinstance(expr, ir.UnaryOp):
        expr.expr = backwards_replace(expr.expr, mapping)
    else:
        raise TypeError(f"{type(expr)} {expr}")
    return expr
