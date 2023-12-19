import logging
import tempfile
import unittest
from importlib import util
from pathlib import Path

import pytest

from python2verilog.api import verilogify
from python2verilog.api.modes import Modes
from python2verilog.api.namespace import (
    get_namespace,
    namespace_to_verilog,
    new_namespace,
)
from python2verilog.api.verilogify import get_actual, get_actual_raw, get_expected
from python2verilog.simulation import iverilog


@pytest.mark.usefixtures("argparse")
class TestSimulation(unittest.TestCase):
    def test_type_hint(self):
        ns = {}

        @verilogify(namespace=ns)
        def func() -> int:
            yield 123

        module, testbench = namespace_to_verilog(ns)
        logging.debug(module)

    def test_type_hint_text(self):
        raw = """
@verilogify
def func() -> int:
    yield 123
        """
        raw = "from python2verilog import verilogify\n" + raw

        # Create a temporary source code file
        with tempfile.NamedTemporaryFile(suffix=".py") as tmp:
            tmp.write(raw.encode())
            tmp.flush()

            # Now load that file as a module
            try:
                spec = util.spec_from_file_location("tmp", tmp.name)
                module = util.module_from_spec(spec)
                spec.loader.exec_module(module)

                # ...or, while the tmp file exists, you can query it externally
                ns = get_namespace(tmp.name)
                logging.debug(ns)
                module, _ = namespace_to_verilog(ns)
                logging.debug(module)

            except Exception as e:
                logging.error(e)

    def test_o0(self):
        ns = {}

        @verilogify(mode=Modes.OVERWRITE, namespace=ns)
        def hrange(n):
            i = 0
            while i < n:
                yield i
                yield i + 100
                i += 1

        @verilogify(
            mode=Modes.OVERWRITE,
            namespace=ns,
            optimization_level=0,
        )
        def dup_range_goal(n):
            inst = hrange(n)
            for i in inst:
                yield i

        list(dup_range_goal(10))

        module, testbench = namespace_to_verilog(ns)
        if self.args.write:
            mod_path = Path(__file__).parent / "o0.sv"
            tb_path = Path(__file__).parent / "o0_tb.sv"
            with open(mod_path, mode="w") as f:
                f.write(str(module))
            with open(tb_path, mode="w") as f:
                f.write(str(testbench))
            cmd = iverilog.make_cmd(
                "dup_range_goal_tb",
                [mod_path, tb_path],
            )
            # warnings.warn(cmd)
        self.assertListEqual(
            list(get_actual(dup_range_goal, module, testbench, timeout=1)),
            list(get_expected(dup_range_goal)),
        )

    def test_o1(self):
        ns = {}

        @verilogify(
            mode=Modes.OVERWRITE,
            namespace=ns,
        )
        def hrange(n):
            i = 0
            while i < n:
                yield i, i
                i += 1

        @verilogify(
            mode=Modes.OVERWRITE,
            namespace=ns,
            optimization_level=1,
        )
        def dup_range_goal(n):
            inst = hrange(n)
            for i, j in inst:
                yield i

        list(dup_range_goal(10))

        module, testbench = namespace_to_verilog(ns)
        if self.args.write:
            mod_path = Path(__file__).parent / "o1.sv"
            tb_path = Path(__file__).parent / "o1_tb.sv"
            with open(mod_path, mode="w") as f:
                f.write(str(module))
            with open(tb_path, mode="w") as f:
                f.write(str(testbench))
            cmd = iverilog.make_cmd(
                "dup_range_goal_tb",
                [mod_path, tb_path],
            )
            # warnings.warn(cmd)
        self.assertListEqual(
            list(get_actual(dup_range_goal, module, testbench, timeout=1)),
            list(get_expected(dup_range_goal)),
        )

    def test_triple0(self):
        """
        Circle lines with -O0
        """
        ns = {}

        @verilogify(namespace=ns, mode=Modes.OVERWRITE)
        def circle_lines(s_x, s_y, height) -> tuple[int, int, int, int, int, int]:
            x = 0
            y = height
            d = 3 - 2 * y
            yield (s_x + x, s_y + y)
            yield (s_x + x, s_y - y)
            yield (s_x - x, s_y + y)
            yield (s_x - x, s_y - y)
            yield (s_x + y, s_y + x)
            yield (s_x + y, s_y - x)
            yield (s_x - y, s_y + x)
            yield (s_x - y, s_y - x)
            while y >= x:
                x = x + 1
                if d > 0:
                    y = y - 1
                    d = d + 4 * (x - y) + 10
                else:
                    d = d + 4 * x + 6
                yield (s_x + x, s_y + y)
                yield (s_x + x, s_y - y)
                yield (s_x - x, s_y + y)
                yield (s_x - x, s_y - y)
                yield (s_x + y, s_y + x)
                yield (s_x + y, s_y - x)
                yield (s_x - y, s_y + x)
                yield (s_x - y, s_y - x)

        @verilogify(namespace=ns, mode=Modes.OVERWRITE, optimization_level=0)
        def triple_circle(centre_x, centre_y, radius):
            # noqa
            c_x = centre_x
            c_y = centre_y
            c_x1 = c_x + radius // 2
            c_y1 = c_y + radius * 2 // 6
            c_x2 = c_x - radius // 2
            c_y2 = c_y + radius * 2 // 6
            c_x3 = c_x
            c_y3 = c_y - radius * 2 // 6

            gen0 = circle_lines(c_x1, c_y1, radius)
            for x, y in gen0:
                yield x, y
            gen0 = circle_lines(c_x3, c_y3, radius)
            for x, y in gen0:
                yield x, y
            gen0 = circle_lines(c_x3, c_y3, radius)
            for x, y in gen0:
                yield x, y

        triple_circle(4, 4, 3)

        module, testbench = namespace_to_verilog(ns)
        if self.args.write:
            mod_path = Path(__file__).parent / "triple_o0.sv"
            tb_path = Path(__file__).parent / "triple_o0_tb.sv"
            with open(mod_path, mode="w") as f:
                f.write(str(module))
            with open(tb_path, mode="w") as f:
                f.write(str(testbench))
            cmd = iverilog.make_cmd(
                "triple_circle_tb",
                [mod_path, tb_path],
            )
            # warnings.warn(cmd)
        self.assertListEqual(
            list(get_actual(triple_circle, module, testbench, timeout=1)),
            list(get_expected(triple_circle)),
        )

    def test_triple(self):
        ns = new_namespace(Path(__file__).parent / "triple_ns")

        @verilogify(namespace=ns, mode=Modes.OVERWRITE)
        def colored_circle(centre_x, centre_y, radius, color):
            offset_y = 0
            offset_x = radius
            crit = 1 - radius
            while offset_y <= offset_x:
                yield (
                    centre_x + offset_x,
                    centre_y + offset_y,
                    color,
                )  # -- octant 1
                yield (centre_x + offset_y, centre_y + offset_x, color)  # -- octant 2
                yield (centre_x - offset_x, centre_y + offset_y, color)  # -- octant 4
                yield (centre_x - offset_y, centre_y + offset_x, color)  # -- octant 3
                yield (centre_x - offset_x, centre_y - offset_y, color)  # -- octant 5
                yield (centre_x - offset_y, centre_y - offset_x, color)  # -- octant 6
                yield (centre_x + offset_x, centre_y - offset_y, color)  # -- octant 8
                yield (centre_x + offset_y, centre_y - offset_x, color)  # -- octant 7
                offset_y = offset_y + 1
                if crit <= 0:
                    crit = crit + 2 * offset_y + 1
                else:
                    offset_x = offset_x - 1
                    crit = crit + 2 * (offset_y - offset_x) + 1

        @verilogify(namespace=ns, mode=Modes.OVERWRITE, optimization_level=1)
        def olympic_logo(mid_x, mid_y, radius):
            spread = radius - 2
            gen = colored_circle(mid_x, mid_y + spread, radius, 50)
            for x, y, color in gen:
                yield x, y, color
            gen = colored_circle(mid_x + spread * 2, mid_y + spread, radius, 180)
            for x, y, color in gen:
                yield x, y, color
            gen = colored_circle(mid_x - spread * 2, mid_y + spread, radius, 500)
            for x, y, color in gen:
                yield x, y, color
            gen = colored_circle(mid_x + spread, mid_y - spread, radius, 400)
            for x, y, color in gen:
                yield x, y, color
            gen = colored_circle(mid_x - spread, mid_y - spread, radius, 300)
            for x, y, color in gen:
                yield x, y, color

        olympic_logo(25, 25, 7)

        # with open("./cyto.log", mode="w") as f:
        #     _, _, cy = context_to_verilog_and_dump(get_context(triple_circle))
        #     f.write(str(cy))
        module, testbench = namespace_to_verilog(ns)
        if self.args.write:
            mod_path = Path(__file__).parent / "olympic.sv"
            tb_path = Path(__file__).parent / "olympic_tb.sv"
            with open(mod_path, mode="w") as f:
                f.write(str(module))
            with open(tb_path, mode="w") as f:
                f.write(str(testbench))
            cmd = iverilog.make_cmd(
                "olympic_logo_tb",
                [mod_path, tb_path],
            )
            # warnings.warn(cmd)
        self.assertListEqual(
            list(get_actual(olympic_logo, module, testbench, timeout=1)),
            list(get_expected(olympic_logo)),
        )

    def test_hrange(self):
        ns = {}

        @verilogify(namespace=ns)
        def hrange(base, limit):
            i = base
            while i < limit:
                yield i
                i += 1

        hrange(0, 10)

        # with open("./cyto.log", mode="w") as f:
        #     _, _, cy = context_to_verilog_and_dump(get_context(triple_circle))
        #     f.write(str(cy))
        module, testbench = namespace_to_verilog(ns)
        if self.args.write:
            mod_path = Path(__file__).parent / "hrange.sv"
            tb_path = Path(__file__).parent / "hrange_tb.sv"
            with open(mod_path, mode="w") as f:
                f.write(str(module))
            with open(tb_path, mode="w") as f:
                f.write(str(testbench))
            cmd = iverilog.make_cmd(
                "hrange_tb",
                [mod_path, tb_path],
            )
            # warnings.warn(cmd)
        self.assertListEqual(
            list(get_actual(hrange, module, testbench, timeout=1)),
            list(get_expected(hrange)),
        )

    def test_reg_func(self):
        ns = {}

        @verilogify(namespace=ns)
        def get_data(addr):
            """
            Dummy function
            """
            print(addr)
            # # Testing reg func that takes more than one clock cycle
            iii = 0
            while iii < addr:
                iii += 1
            return iii
            # return addr + 42069

        @verilogify(namespace=ns)
        def read32to8(base, count):
            i = 0
            while i < count:
                data = get_data(base + count * 4)
                j = 0
                while j < 4:
                    yield data
                    j += 1
                i += 1

        read32to8(256, 2)

        # with open("./cyto.log", mode="w") as f:
        #     _, _, cy = context_to_verilog_and_dump(get_context(triple_circle))
        #     f.write(str(cy))
        module, testbench = namespace_to_verilog(ns)
        if self.args.write:
            mod_path = Path(__file__).parent / "read32to8.sv"
            tb_path = Path(__file__).parent / "read32to8_tb.sv"
            with open(mod_path, mode="w") as f:
                f.write(str(module))
            with open(tb_path, mode="w") as f:
                f.write(str(testbench))
            cmd = iverilog.make_cmd(
                "read32to8_tb",
                [mod_path, tb_path],
            )
        self.assertListEqual(
            list(get_actual(read32to8, module, testbench, timeout=1)),
            list(get_expected(read32to8)),
        )

    def test_reg_func2(self):
        ns = {}

        @verilogify(namespace=ns)
        def get_data(addr):
            """
            Dummy function
            """
            print(addr)
            return addr + 42069

        @verilogify(namespace=ns)
        def read32to8_2(base, count):
            i = 0
            while i < count:
                data = get_data(base + count * 4)
                j = 0
                while j < 4:
                    yield data
                    j += 1
                i += 1

        read32to8_2(256, 2)

        # with open("./cyto.log", mode="w") as f:
        #     _, _, cy = context_to_verilog_and_dump(get_context(triple_circle))
        #     f.write(str(cy))
        module, testbench = namespace_to_verilog(ns)
        if self.args.write:
            mod_path = Path(__file__).parent / "read32to8_2.sv"
            tb_path = Path(__file__).parent / "read32to8_2_tb.sv"
            with open(mod_path, mode="w") as f:
                f.write(str(module))
            with open(tb_path, mode="w") as f:
                f.write(str(testbench))
            cmd = iverilog.make_cmd(
                "read32to8_2_tb",
                [mod_path, tb_path],
            )
        self.assertListEqual(
            list(get_actual(read32to8_2, module, testbench, timeout=1)),
            list(get_expected(read32to8_2)),
        )
