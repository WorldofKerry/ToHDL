import unittest

from python2verilog.utils.lines import Indent, Lines


class TestLines(unittest.TestCase):
    def test_constructor(self):
        lines = Lines()
        self.assertEqual("", lines.to_string())
        self.assertEqual("", str(lines))

        lines = Lines("abc 123")
        self.assertEqual("abc 123\n", str(lines))
        self.assertEqual("abc 123\n", str(Lines("abc 123\n")))
        self.assertEqual("\n", str(Lines("\n")))
        self.assertRaises(AssertionError, Lines, {})

        lines = Lines(["abc 123"])
        self.assertEqual("abc 123\n", str(lines))
        self.assertRaises(AssertionError, Lines, ["abc 123\n"])
        self.assertRaises(AssertionError, Lines, ["\n"])

        lines = Lines(["abc 123", "def 456"])
        self.assertEqual("abc 123\ndef 456\n", str(lines))
        self.assertRaises(AssertionError, Lines, ["abc 123\n", "def 456"])
        self.assertRaises(AssertionError, Lines, ["abc 123", "def 456\n"])

    def test_operations(self):
        lines = Lines("abc 123")
        lines += "def 456"
        self.assertEqual("abc 123\ndef 456\n", str(lines))
        del lines[-1]
        self.assertEqual("abc 123\n", str(lines))
        lines.concat(Lines("ghi 789"))
        self.assertEqual("abc 123\nghi 789\n", str(lines))
        self.assertRaises(AssertionError, lines.concat, "abc 123")

        for i in range(3):
            lines = Lines("abc 123")
            self.assertEqual(Indent(i) + "abc 123\n", str(lines.indent(i)))

        lines = Lines("abc 123")
        sum = 0
        for i in range(3):
            sum += i
            self.assertEqual(Indent(sum) + "abc 123\n", str(lines.indent(i)))


class TestIndent(unittest.TestCase):
    def test_all(self):
        self.assertEqual("", str(Indent()))

        for i in range(3):
            self.assertEqual(Indent.indentor * i, str(Indent(i)))
            self.assertEqual(Indent.indentor * i + "abc 123", Indent(i) + "abc 123")
            self.assertEqual("abc 123" + Indent.indentor * i, "abc 123" + Indent(i))
            self.assertRaises(AssertionError, lambda: i + Indent(i))
            self.assertRaises(AssertionError, lambda: Indent(i) + None)
