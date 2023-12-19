"""
Verilog Abstract Syntax Tree Components
"""

from __future__ import annotations

from typing import Optional

from python2verilog import ir
from python2verilog.utils import env
from python2verilog.utils.generics import GenericRepr
from python2verilog.utils.lines import ImplementsToLines, Indent, Lines
from python2verilog.utils.typed import guard, typed, typed_list, typed_strict


class AtPosedge(ir.Expression):
    """
    @(posedge <condition>)
    """

    def __init__(self, condition: ir.Expression):
        assert isinstance(condition, ir.Expression)
        self.condition = condition
        super().__init__(f"@(posedge {condition.to_string()})")


class AtNegedge(ir.Expression):
    """
    @(negedge <condition>)
    """

    def __init__(self, condition: ir.Expression):
        assert isinstance(condition, ir.Expression)
        self.condition = condition
        super().__init__(f"@(negedge {condition.to_string()})")


class Statement(ImplementsToLines, GenericRepr):
    """
    Represents a statement in verilog (i.e. a line or a block)
    If used directly, it is treated as a string literal
    """

    def __init__(self, literal: str = "", comment: str = ""):
        self.literal = typed(literal, str)
        self.comment = typed(comment, str)

    def to_lines(self):
        """
        To Verilog
        """
        if self.literal:
            return Lines(self.literal + self.get_inline_comment())
        return Lines(self.get_inline_comment()[1:])  # removes leading space

    def get_inline_comment(self):
        """
        // <comment>
        """
        if self.comment != "":
            return f" // {self.comment}"
        return ""

    def get_blocked_comment(self):
        """
        // <comment>
        ...
        // <comment>
        Separated by newlines
        """
        assert guard(self.comment, str)
        actual_lines = self.comment.split("\n")
        lines = Lines()
        for line in actual_lines:
            lines += f"// {line}"
        return lines


class LocalParam(Statement):
    """
    localparam <name> = <value>;
    """

    def __init__(self, name: str, value: ir.UInt, *args, **kwargs):
        assert isinstance(value, ir.UInt)
        super().__init__(f"localparam {name} = {value.verilog()};", *args, **kwargs)


class TypeDef(Statement):
    """
    typedef enum
    {
        <val0>, <val1>, ...
    } _state_t;
    """

    def __init__(self, name: str, values: list[str]):
        self.name = typed(name, str)
        self.values = typed_list(values, str)
        super().__init__()

    def to_lines(self):
        lines = Lines("typedef enum")
        lines += "{"
        values = Lines()
        for value in self.values[:-1]:
            values += f"{value},"
        values += f"{self.values[-1]}"
        lines.concat(values, indent=1)
        lines += f"}} {self.name};"

        # condense
        lines = Lines(lines.to_string().replace("\n", "").replace("  ", ""))
        return lines


class AtPosedgeStatement(Statement):
    """
    @(posedge <condition>);
    """

    def __init__(self, condition: ir.Expression, *args, **kwargs):
        assert isinstance(condition, ir.Expression)
        super().__init__(f"{AtPosedge(condition).to_string()};", *args, **kwargs)


class AtNegedgeStatement(Statement):
    """
    @(negedge <condition>);
    """

    def __init__(self, condition: ir.Expression, *args, **kwargs):
        assert isinstance(condition, ir.Expression)
        super().__init__(f"{AtNegedge(condition).to_string()};", *args, **kwargs)


class Instantiation(Statement):
    """
    Instantiationo f Verilog module.
    <module-name> <given-name> (...);
    """

    def __init__(
        self,
        module_name: str,
        given_name: str,
        port_connections: dict[str, str],
        *args,
        **kwargs,
    ):
        """
        port_connections[port_name] = signal_name, i.e. `.port_name(signal_name)`
        """
        assert isinstance(given_name, str)
        assert isinstance(module_name, str)
        for key, val in port_connections.items():
            assert isinstance(key, str)
            assert isinstance(val, str)
        self.given_name = given_name
        self.module_name = module_name
        self.port_connections = port_connections
        super().__init__(*args, **kwargs)

    def to_lines(self):
        """
        To Verilog
        """
        lines = Lines()
        lines += f"{self.module_name} {self.given_name} ("
        for key, val in self.port_connections.items():
            lines += Indent(1) + f".{key}({val}),"
        lines[-1] = lines[-1][:-1]  # Remove last comma
        lines += Indent(1) + ");"
        return lines


class Module(ImplementsToLines):
    """
    module name(...); endmodule
    """

    def __init__(
        self,
        name: str,
        body: Optional[list[Statement]] = None,
        localparams: Optional[dict[str, ir.UInt]] = None,
        header: Optional[Lines] = None,
    ):
        self.name = name

        self.inputs = Lines()
        self.outputs = Lines()

        if body:
            for stmt in body:
                assert isinstance(stmt, Statement), f"got {type(stmt)} instead"
            self.body = body
        else:
            self.body = []

        if localparams:
            if env.get_var(env.Vars.IS_SYSTEM_VERILOG) is not None:
                self.local_params = Lines("// State variables")
                self.local_params.concat(
                    TypeDef("_state_t", list(localparams.keys())).to_lines()
                )
                self.local_params += "_state_t _state;"
            else:
                self.local_params = Lines("// State variables")
                for key, value in localparams.items():
                    self.local_params.concat(LocalParam(key, value).to_lines())
                self.local_params.concat(Declaration("_state", reg=True).to_lines())
        else:
            self.local_params = Lines()

        self.header_comment = header

    def to_lines(self):
        """
        To Verilog
        """
        if self.header_comment:
            lines = Lines(self.header_comment.lines)
        else:
            lines = Lines()
        lines += f"module {self.name} ("
        lines.concat(self.inputs, indent=1)
        lines.concat(self.outputs, indent=1)

        if self.inputs or self.outputs:  # This means there are ports
            lines[-1] = lines[-1][0:-1]  # removes last comma

        lines += ");"
        lines.concat(self.local_params, 1)
        for stmt in self.body:
            lines.concat(stmt.to_lines(), 1)
        lines += "endmodule"
        lines.blank()
        return lines


class Initial(Statement):
    """
    initial begin
        ...
    end
    """

    def __init__(self, *args, body: Optional[list[Statement]] = None, **kwargs):
        if body:
            typed_list(body, Statement)
        self.body = body
        super().__init__(*args, **kwargs)

    def to_lines(self):
        lines = Lines("initial begin")
        if self.body:
            for stmt in self.body:
                lines.concat(stmt.to_lines(), 1)
        lines += "end"
        return lines


class Always(Statement):
    """
    always () begin
        ...
    end
    """

    def __init__(
        self,
        trigger: ir.Expression,
        *args,
        body: Optional[list[Statement]] = None,
        **kwargs,
    ):
        assert isinstance(trigger, ir.Expression)
        self.trigger = trigger
        if body:
            for stmt in body:
                assert isinstance(stmt, Statement)
            self.body = body
        else:
            self.body = []
        super().__init__(*args, **kwargs)

    def to_lines(self):
        """
        To Verilog
        """
        lines = Lines(f"always {self.trigger.verilog()} begin")
        for stmt in self.body:
            lines.concat(stmt.to_lines(), 1)
        lines += "end"
        return lines


class PosedgeSyncAlways(Always):
    """
    always @(posedge <clock>) begin
        <valid> = 0;
    end
    """

    def __init__(self, clock: ir.Expression, *args, **kwargs):
        assert isinstance(clock, ir.Expression)
        self.clock = clock
        super().__init__(AtPosedge(clock), *args, **kwargs)


class Subsitution(Statement):
    """
    Interface for
    <lvalue> <blocking or nonblocking> <rvalue>
    """

    def __init__(
        self,
        lvalue: ir.Var,
        rvalue: ir.Expression,
        oper: str,
        *args,
        **kwargs,
    ):
        assert isinstance(rvalue, (ir.Expression)), f"got {type(rvalue)} instead"
        assert isinstance(lvalue, (ir.Expression)), f"{lvalue}"
        self.lvalue = lvalue
        self.rvalue = rvalue
        self.oper = oper

        super().__init__(*args, **kwargs)

    def to_lines(self):
        """
        Converts to Verilog
        """
        assert isinstance(self.oper, str), "Subclasses need to set self.type"
        self.literal = f"{self.lvalue.verilog()} {self.oper} {self.rvalue.verilog()};"
        return super().to_lines()


class NonBlockingSubsitution(Subsitution):
    """
    <lvalue> <= <rvalue>
    """

    def __init__(self, lvalue: ir.Var, rvalue: ir.Expression, *args, **kwargs):
        super().__init__(lvalue, rvalue, "<=", *args, **kwargs)


class BlockingSub(Subsitution):
    """
    <lvalue> = <rvalue>
    """

    def __init__(self, lvalue: ir.Var, rvalue: ir.Expression, *args, **kwargs):
        super().__init__(lvalue, rvalue, "=", *args, **kwargs)


class Declaration(Statement):
    """
    <reg or wire> <modifiers> <[size-1:0]> <name>;
    """

    def __init__(
        self,
        name: str,
        *args,
        size: int = 32,
        reg: bool = False,
        signed: bool = False,
        **kwargs,
    ):
        self.size = typed_strict(size, int)
        self.reg = typed_strict(reg, bool)
        self.signed = typed_strict(signed, bool)
        self.name = typed_strict(name, str)
        super().__init__(*args, **kwargs)

    def to_lines(self):
        """
        To Verilog lines
        """
        string = ""
        if self.reg:
            string += "reg"
        else:
            string += "wire"
        if self.signed:
            string += " signed"
        if self.size > 1:
            string += f" [{self.size-1}:0]"
        string += f" {self.name}"
        string += ";"
        return Lines(string)


class CaseItem(ImplementsToLines):
    """
    Verilog case item, i.e.
    <condition>: begin
        <statements>
    end
    """

    def __init__(self, condition: ir.Expression, statements: list[Statement]):
        assert isinstance(condition, ir.Expression)
        self.condition = condition  # Can these by expressions are only literals?
        if statements:
            for stmt in statements:
                assert isinstance(stmt, Statement), f"unexpected {type(stmt)}"
            self.statements = statements
        else:
            self.statements = []

    def to_lines(self):
        """
        To Verilog lines
        """
        lines = Lines()
        lines += f"{self.condition.to_string()}: begin"
        for stmt in self.statements:
            lines.concat(stmt.to_lines(), indent=1)
        lines += "end"
        return lines


class Case(Statement):
    """
    Verilog case statement with various cases
    case (<expression>)
        <items[0]>
        ...
        <items[n]>
    endcase
    """

    def __init__(
        self, expression: ir.Expression, case_items: list[CaseItem], *args, **kwargs
    ):
        assert isinstance(expression, ir.Expression)
        self.condition = expression
        if case_items:
            for item in case_items:
                assert isinstance(item, CaseItem)
            self.case_items = case_items
        else:
            self.case_items = []
        super().__init__(*args, **kwargs)

    def to_lines(self):
        """
        To Verilog Lines
        """
        if len(self.case_items) == 0:
            return Lines(f"// case({self.condition})\n//Empty case block\n// endcase")
        lines = Lines()
        lines += f"case ({self.condition.to_string()})"
        for item in self.case_items:
            lines.concat(item.to_lines(), indent=1)
        lines += "endcase"
        return lines


class IfElse(Statement):
    """
    Verilog if else
    """

    def __init__(
        self,
        condition: ir.Expression,
        then_body: list[Statement],
        else_body: Optional[list[Statement]],
        *args,
        **kwargs,
    ):
        super().__init__(*args, **kwargs)
        self.condition = typed_strict(condition, ir.Expression)
        self.then_body = typed_list(then_body, Statement)
        self.else_body = typed_list(else_body, Statement)

    def to_lines(self):
        lines = Lines()
        lines += f"if ({self.condition.verilog()}) begin" + (
            f" // {self.comment}" if self.comment else ""
        )
        for stmt in self.then_body:
            lines.concat(stmt.to_lines(), indent=1)
        if self.else_body:
            lines += "end else begin"
            for stmt in self.else_body:
                lines.concat(stmt.to_lines(), indent=1)
        lines += "end"
        return lines


class While(Statement):
    """
    Unsynthesizable While
    while (<condition>) begin
        ...
    end
    """

    def __init__(
        self,
        *args,
        condition: ir.Expression,
        body: Optional[list[Statement]] = None,
        **kwargs,
    ):
        assert isinstance(condition, ir.Expression)
        self.condition = condition
        if body:
            typed_list(body, Statement)
        self.body = body
        super().__init__(*args, **kwargs)

    def to_lines(self):
        lines = Lines(f"while ({self.condition.to_string()}) begin")
        if self.body:
            for stmt in self.body:
                lines.concat(stmt.to_lines(), 1)
        lines += "end"
        return lines
