"""
Creates module from context and FSM
"""

from typing import Iterator, cast

from python2verilog import ir
from python2verilog.backend.verilog import ast as ver
from python2verilog.ir.expressions import UInt
from python2verilog.optimizer.helpers import backwards_replace
from python2verilog.utils.lines import Lines


class Module(ver.Module):
    """
    A module that implements the python2verilog module interface
    """

    def __init__(self, context: ir.Context, root: ver.Case):
        """
        Creates a module wrapper from the context

        Requires context for I/O and declarations
        """
        assert isinstance(root, ver.Case)
        assert isinstance(context, ir.Context)

        inputs: list[str] = []
        for var in context.input_vars:
            inputs.append(var.py_name)

        outputs = []
        for var in context.output_vars:
            outputs.append(var.ver_name)

        def make_debug_display(context: ir.Context):
            """
            Creates a display statement for all signals

            $display("%0d, ...", ...);
            """
            vars_: list[str] = []
            vars_ += map(
                lambda x: x.ver_name, context.signals.instance_specific_values()
            )
            vars_ += map(lambda x: x.py_name, context.input_vars)  # module inputs
            vars_ += map(lambda x: x.ver_name, context.input_vars)  # cache
            vars_ += map(lambda x: x.ver_name, context.output_vars)
            vars_ += map(lambda x: x.ver_name, context.local_vars)
            str_ = f'$display("{context.name},%s,'
            str_ += "=%0d,".join(vars_) + '=%0d", '
            str_ += f"{context.state_var.ver_name}.name, "
            str_ += ", ".join(vars_)
            str_ += ");"
            return str_

        def create_instance_zeroed_signals() -> Iterator[ver.Statement]:
            """
            Instance signals that should always be set to zero be default
            """
            for instance in context.generator_instances.values():
                yield ver.NonBlockingSubsitution(instance.signals.ready, ir.UInt(0))
                yield ver.NonBlockingSubsitution(instance.signals.start, ir.UInt(0))

        always_body = (
            [
                ver.Statement("`ifdef DEBUG"),
                ver.Statement(make_debug_display(context)),
                ver.Statement("`endif"),
                ver.Statement(),
            ]
            + list(create_instance_zeroed_signals())
            + [
                ver.Statement(),
                ver.IfElse(
                    context.signals.ready,
                    cast(
                        list[ver.Statement],
                        [
                            ver.NonBlockingSubsitution(
                                context.signals.valid, ir.UInt(0)
                            ),
                            ver.NonBlockingSubsitution(
                                context.signals.done, ir.UInt(0)
                            ),
                        ],
                    ),
                    [],
                ),
            ]
            + [
                ver.Statement(),
                ver.Statement(comment="Start signal takes precedence over reset"),
                ver.IfElse(
                    ir.UBinOp(context.signals.reset, "||", context.signals.start),
                    then_body=[
                        ver.NonBlockingSubsitution(
                            context.state_var,
                            context.idle_state,
                        ),
                        ver.NonBlockingSubsitution(
                            context.signals.done,
                            ir.UInt(0),
                        ),
                        ver.NonBlockingSubsitution(
                            context.signals.valid,
                            ir.UInt(0),
                        ),
                    ],
                    else_body=[],
                ),
                ver.Statement(),
            ]
            + Module.make_start_ifelse(root, context)
        )

        always = ver.PosedgeSyncAlways(clock=context.signals.clock, body=always_body)

        module_body: list[ver.Statement] = []

        module_body += [
            ver.Statement(comment="Local variables"),
        ]
        context.local_vars.sort(key=lambda x: x.ver_name)
        module_body += [
            ver.Declaration(v.ver_name, reg=True, signed=True)
            for v in context.local_vars
        ]

        module_body += [
            ver.Declaration(var.ver_name, reg=True, signed=True)
            for var in context.input_vars
        ]

        for instance in context.generator_instances.values():
            module_body.append(
                ver.Statement(
                    comment="================ Function Instance ================"
                )
            )
            module = context.namespace[instance.module_name]
            defaults: dict[ir.Var, ir.Var] = {
                module.signals.valid: instance.signals.valid,
                module.signals.done: instance.signals.done,
                module.signals.clock: context.signals.clock,
                module.signals.start: instance.signals.start,
                module.signals.reset: instance.signals.reset,
                module.signals.ready: instance.signals.ready,
            }
            for var in instance.inputs:
                module_body.append(ver.Declaration(name=var.ver_name, reg=True))
            for var in instance.outputs:
                module_body.append(ver.Declaration(name=var.ver_name))
            module_body.append(
                ver.Declaration(name=instance.signals.valid.ver_name, size=1)
            )
            module_body.append(
                ver.Declaration(name=instance.signals.done.ver_name, size=1)
            )
            module_body.append(
                ver.Declaration(name=instance.signals.start.ver_name, size=1, reg=True)
            )
            module_body.append(
                ver.Declaration(name=instance.signals.ready.ver_name, size=1, reg=True)
            )
            module_body.append(
                ver.Instantiation(
                    instance.module_name,
                    instance.var.ver_name,
                    {
                        key.py_name: value.ver_name
                        for key, value in zip(
                            module.input_vars,
                            instance.inputs,
                        )
                    }
                    | {
                        key.ver_name: value.ver_name
                        for key, value in zip(
                            module.output_vars,
                            instance.outputs,
                        )
                    }
                    | {key.ver_name: value.ver_name for key, value in defaults.items()},
                )
            )

        module_body.append(ver.Statement(comment="Core"))
        module_body.append(always)

        # Consistent state var ordering in transpile
        state_vars = {
            key: ir.UInt(index) for index, key in enumerate(sorted(context.states))
        }

        super().__init__(
            name=context.name,
            body=module_body,
            localparams=state_vars,
        )

        self.inputs = self._input_lines(inputs=inputs, context=context)
        self.outputs = self._output_lines(outputs=outputs, context=context)

    @staticmethod
    def _input_lines(inputs: list[str], context: ir.Context):
        """
        Module inputs
        """
        input_lines = Lines()
        input_lines += (
            "// Function parameters (only need to be set when start is high):"
        )
        for input_ in inputs:
            assert isinstance(input_, str)
            input_lines += f"input wire signed [31:0] {input_},"
        input_lines.blank()
        input_lines += (
            f"input wire {context.signals.clock.ver_name}, " "// clock for sync"
        )
        input_lines += (
            f"input wire {context.signals.reset.ver_name}, "
            "// set high to reset, i.e. done will be high"
        )
        input_lines += (
            f"input wire {context.signals.start}, "
            + "// set high to capture inputs (in same cycle) and start generating"
        )
        input_lines.blank()
        input_lines += "// Implements the ready/valid handshake"
        input_lines += (
            f"input wire {context.signals.ready}, "
            "// set high when caller is ready for output"
        )
        return input_lines

    @staticmethod
    def _output_lines(outputs: list[str], context: ir.Context):
        """
        Get outputs
        """
        output_lines = Lines()
        output_lines += (
            f"output reg {context.signals.valid}, "
            "// is high if output values are valid"
        )
        output_lines.blank()
        output_lines += (
            f"output reg {context.signals.done}, "
            "// is high if module done outputting"
        )
        output_lines.blank()
        output_lines += "// Output values as a tuple with respective index(es)"
        for output in outputs:
            assert isinstance(output, str)
            output_lines += f"output reg signed [31:0] {output},"
        return output_lines

    @staticmethod
    def make_start_ifelse(root: ver.Case, context: ir.Context) -> list[ver.Statement]:
        """
        if (_start) begin
            ...
        end else begin
            ...
        end
        """
        then_body: list[ver.Statement] = []
        for var in context.input_vars:
            then_body.append(
                ver.NonBlockingSubsitution(
                    ir.Var(py_name=var.ver_name, ver_name=var.ver_name),
                    ir.Expression(var.py_name),
                )
            )

        if context.optimization_level > 0:
            # Optimization to include the entry state in the start ifelse

            # Map cached inputs to input signals (cached inputs not updated yet)
            mapping = {
                ir.Var(py_name=var.ver_name, ver_name=var.ver_name): ir.Expression(
                    var.py_name
                )
                for var in context.input_vars
            }

            # Get statements in entry state
            stmt_stack: list[ver.Statement] = []
            for item in root.case_items:
                if item.condition == context.entry_state:
                    stmt_stack += item.statements
                    then_body += item.statements
                    root.case_items.remove(item)
                    break

            # Replace usage of cached inputs with input signals
            while stmt_stack:
                stmt = stmt_stack.pop()
                if isinstance(stmt, ver.NonBlockingSubsitution):
                    stmt.rvalue = backwards_replace(stmt.rvalue, mapping)
                elif isinstance(stmt, ver.IfElse):
                    stmt.condition = backwards_replace(stmt.condition, mapping)
                    stmt_stack += stmt.then_body
                    stmt_stack += stmt.else_body
                else:
                    raise TypeError(f"Unexpected {type(stmt)} {stmt}")
        else:
            then_body.append(
                ver.NonBlockingSubsitution(context.state_var, context.entry_state)
            )

        if_else = ver.IfElse(
            context.signals.start,
            then_body,
            [
                ver.Statement(
                    comment="If ready or not valid, then continue computation"
                ),
                ver.IfElse(
                    ir.UBinOp(
                        context.signals.ready,
                        "||",
                        ir.UnaryOp("!", context.signals.valid),
                    ),
                    [root],
                    [],
                ),
            ],
        )
        return [if_else]
