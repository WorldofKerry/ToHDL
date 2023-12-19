"""
Parses Verilog display statements
"""

from typing import Iterable, Iterator, Union

from python2verilog.exceptions import UnknownValueError


def parse_stdout(stdout: str) -> Iterator[tuple[str, ...]]:
    """
    Implementation-specific (based on testbench output)
    Yields the signals and outputs for a display statement
    :return: [valid, ready, output0, output1, ...]
    """
    for line in stdout.splitlines():
        yield tuple(elem.strip() for elem in line.split(","))


def __strip_and_filter_first_signal(
    actual: Iterable[Union[tuple[str, ...], str]],
    length_check: int,
) -> Iterator[Union[tuple[str, ...], str]]:
    """
    Filters based on row[0]

    Removes tailing messages, e.g. `$finish`.

    :return: row[1:]
    """
    for row in actual:
        if len(row) > length_check:
            if row[0] == "1":
                outputs = row[1:]
                if len(outputs) == 1:
                    yield outputs[0]
                else:
                    yield tuple(elem for elem in outputs)
        else:
            if "$finish" in " ".join(row):
                pass
            else:
                raise ValueError(f"Unexpected row {row}")


def strip_ready(
    actual: Iterable[Union[tuple[str, ...], str]]
) -> Iterator[Union[tuple[str, ...], str]]:
    """
    Assumes first two signals to be ready and valid,
    such that a row is [ready, valid, output0, output1, ...]

    Removes tailing messages, e.g. `$finish`.

    :return: [valid, output0, output1, ...]
    """
    yield from __strip_and_filter_first_signal(actual, 2)


def strip_valid(
    actual: Iterable[Union[tuple[str, ...], str]]
) -> Iterator[Union[tuple[int, ...], int]]:
    """
    Assumes first signal to be valid,
    such that a row is [valid, output0, output1, ...]

    Throws if ready, but an output is 'x'.

    Removes tailing messages, e.g. `$finish`.

    :return: [output0, output1, ...]
    """
    for row in __strip_and_filter_first_signal(actual, 1):
        try:
            if isinstance(row, tuple) and len(row) > 1:
                yield tuple(int(e) for e in row)
            elif isinstance(row, str):
                yield int(row)  # otherwise take first char of string
            else:
                yield int(row[0])
        except ValueError as e:
            raise UnknownValueError(f"Unknown logic value in outputs {row}") from e
