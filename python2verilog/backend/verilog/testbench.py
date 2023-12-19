"""
Creates testbench from context and FSM
"""
import itertools
import logging

from python2verilog import ir
from python2verilog.backend.verilog import ast as ver
from python2verilog.backend.verilog.ast import Statement
from python2verilog.backend.verilog.config import TestbenchConfig
from python2verilog.optimizer.helpers import backwards_replace
from python2verilog.utils.lines import Lines
from python2verilog.utils.typed import guard_dict


class Testbench(ver.Module):
    """
    Testbench
    """

    def __init__(self, context: ir.Context, config: TestbenchConfig):
        # pylint: disable=too-many-locals
        self.context = context
        logging.debug("%s", config)

        def make_display_stmt():
            """
            Creates a display statement for protocol signals and outputs

            $display("%0d, ...", ...);
            """
            string = '$display("%0d, %0d, '
            string += "%0d, " * (len(self.context.output_vars) - 1)
            string += f'%0d", {context.signals.ready.ver_name}, {context.signals.valid.ver_name}'
            for var in self.context.output_vars:
                string += f", {var.ver_name}"
            string += ");"
            return ver.Statement(literal=string)

        assert isinstance(self.context, ir.Context)
        decl: list[ver.Declaration] = []
        decl.append(ver.Declaration(context.signals.clock.ver_name, size=1, reg=True))
        decl.append(ver.Declaration(context.signals.start.ver_name, size=1, reg=True))
        decl.append(ver.Declaration(context.signals.reset.ver_name, size=1, reg=True))
        decl.append(ver.Declaration(context.signals.ready.ver_name, size=1, reg=True))
        decl += [
            ver.Declaration(var.py_name, signed=True, reg=True)
            for var in self.context.input_vars
        ]
        decl.append(ver.Declaration(context.signals.done.ver_name, size=1))
        decl.append(ver.Declaration(context.signals.valid.ver_name, size=1))
        decl += [
            ver.Declaration(var.ver_name, signed=True)
            for var in self.context.output_vars
        ]

        ports = {decl.name: decl.name for decl in decl}
        assert guard_dict(ports, str, str)

        setups: list[ver.Statement] = list(decl)
        setups.append(ver.Instantiation(self.context.name, "DUT", ports))

        setups.append(
            ver.Statement(
                literal=f"always #5 {context.signals.clock.ver_name} = "
                f"!{context.signals.clock.ver_name};"
            )
        )
        if config.random_ready:
            random_wait_counter = ir.Var("random_wait_counter")
            setups.append(ver.Declaration(random_wait_counter.ver_name, reg=True))

        initial_body: list[ver.Statement | ver.While] = []
        initial_body.append(ver.BlockingSub(self.context.signals.clock, ir.UInt(0)))
        initial_body.append(ver.BlockingSub(self.context.signals.start, ir.UInt(0)))
        initial_body.append(ver.BlockingSub(self.context.signals.ready, ir.UInt(1)))
        initial_body.append(ver.BlockingSub(self.context.signals.reset, ir.UInt(1)))

        initial_body.append(ver.AtNegedgeStatement(self.context.signals.clock))
        initial_body.append(ver.BlockingSub(self.context.signals.reset, ir.UInt(0)))
        if config.random_ready:
            initial_body.append(ver.BlockingSub(random_wait_counter, ir.Int(8)))

        initial_body.append(ver.Statement())

        logging.debug("Making test cases")
        for i, test_case in enumerate(self.context.test_cases):
            # New test case and start
            initial_body.append(
                ver.Statement(
                    comment=f"============ Test Case {i} with "
                    f"arguments {str(test_case)} ============"
                )
            )
            for i, var in enumerate(self.context.input_vars):
                initial_body.append(
                    ver.BlockingSub(
                        ir.Var(py_name=var.py_name, ver_name=var.py_name),
                        ir.Int(int(test_case[i])),
                    )
                )
            initial_body.append(ver.BlockingSub(self.context.signals.start, ir.UInt(1)))

            # Post-start
            initial_body.append(ver.Statement())
            initial_body.append(ver.AtNegedgeStatement(self.context.signals.clock))
            for i, var in enumerate(self.context.input_vars):
                initial_body.append(
                    ver.BlockingSub(
                        ir.Var(py_name=var.py_name, ver_name=var.py_name),
                        ir.Unknown(),
                        comment="only need inputs when start is set",
                    )
                )
            initial_body.append(ver.BlockingSub(self.context.signals.start, ir.UInt(0)))
            initial_body.append(ver.Statement())

            # While loop
            while_body: list[ver.Statement] = []
            while_body.append(ver.AtPosedgeStatement(self.context.signals.clock))
            then_body = [make_display_stmt()]
            if config.random_ready:
                then_body.append(ver.BlockingSub(random_wait_counter, ir.Int(4)))
            while_body.append(
                ver.IfElse(
                    condition=self.context.signals.ready,
                    then_body=then_body,
                    else_body=[],
                )
            )
            while_body.append(ver.AtNegedgeStatement(self.context.signals.clock))
            if config.random_ready:
                while_body.append(
                    ver.BlockingSub(
                        random_wait_counter,
                        ir.BinOp(random_wait_counter, "-", ir.Int(1)),
                    )
                )
                while_body.append(
                    ver.BlockingSub(
                        self.context.signals.ready,
                        ir.BinOp(random_wait_counter, "===", ir.Int(0)),
                    )
                )
            initial_body.append(
                ver.While(
                    condition=ir.UBinOp(
                        ir.UnaryOp(
                            "!",
                            ir.BinOp(
                                self.context.signals.done,
                                "&&",
                                self.context.signals.valid,
                            ),
                        ),
                        "||",
                        ir.UnaryOp("!", self.context.signals.ready),
                    ),
                    body=while_body,
                )
            )

            if not self.context.is_generator:
                initial_body.append(
                    ver.IfElse(
                        condition=self.context.signals.ready,
                        then_body=[make_display_stmt()],
                        else_body=[],
                    )
                )

            initial_body.append(ver.Statement())

        initial_body.append(ver.Statement(literal="$finish;"))

        initial_loop = ver.Initial(body=initial_body)

        logging.debug("Creating python test code")
        python_test_code = Lines()
        for case in self.context.test_cases:
            python_test_code += f"print(list({self.context.name}(*{case})))"

        super().__init__(
            self.context.testbench_name,
            body=setups + [initial_loop],
            header=Lines(
                f"/*\n# Python Function\n{self.context.py_string}\n"
                f"# Test Cases\n{python_test_code}*/\n"
            ),
        )
