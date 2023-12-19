"""
Icarious Verilog CLI Abstractions
"""

import logging
import os
import signal
import subprocess
import tempfile
import time
from typing import Iterable, Optional, Union

from python2verilog.utils import env
from python2verilog.utils.typed import guard_dict


def make_cmd(top_level_module: str, files: Iterable[Union[str, os.PathLike[str]]]):
    """
    Returns an iverilog command

    :param files: set of absolute paths to files required for simulation
    """
    cmd = (
        f"{env.get_var(env.Vars.IVERILOG_PATH)} -g2005-sv -Wall -s {top_level_module} "
    )
    cmd += " ".join(map(str, files)) + " "
    # pylint: disable=consider-using-with
    cache_file = tempfile.NamedTemporaryFile(mode="r", encoding="utf8")
    cache_path = cache_file.name
    cmd += f"-o {cache_path} && unbuffer vvp {cache_path} && rm {cache_path}"
    return cmd


def _write_data_to_paths(path_to_data: dict[str, str]):
    """
    Writes data to respective path
    """
    logging.debug("Writing data to paths start")
    for path, data in path_to_data.items():
        with open(path, mode="w", encoding="utf8") as file:
            logging.debug("Writing %s to %s", len(data), file.name)
            file.write(data)
    logging.debug("Writing data to paths done")


def _run_cmd_with_fifos(
    command: str, input_fifos: dict[str, str], timeout: Optional[int] = None
):
    """
    Runs a command that uses fifos as input. DO NOT use this function with regular files.

    :param input_fifos: absolute file path of fifos -> data to write

    :return: (stdout, stderr/exception)
    """
    guard_dict(input_fifos, str, str)  # type: ignore
    # pylint: disable=subprocess-popen-preexec-fn
    with subprocess.Popen(
        command,
        shell=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        preexec_fn=os.setsid,
    ) as process:
        _write_data_to_paths(input_fifos)

        try:
            logging.debug("Waiting on process for %ss", timeout)
            start_time = time.time()
            process.wait(timeout=timeout)
            logging.debug("Took %ss", time.time() - start_time)
            assert process.stdout
            assert process.stderr
            return process.stdout.read(), process.stderr.read()
        except subprocess.TimeoutExpired as e:
            assert process.stdout
            assert process.stderr
            stdout = process.stdout.read()
            stderr = process.stderr.read()
            logging.debug("%s, %s, %s", e, stdout, stderr)
            os.killpg(os.getpgid(process.pid), signal.SIGTERM)
            return stdout, stderr


def _run_cmd_with_files(
    command: str, input_files: dict[str, str], timeout: Optional[int] = None
):
    """
    Runs a command that uses files as input. DO NOT use this function with regular fifos.

    :param input_files: absolute file path of files -> data to write

    :return: (stdout, stderr/exception)
    """
    guard_dict(input_files, str, str)  # type: ignore
    # _write_data_to_paths(input_files)
    # pylint: disable=subprocess-popen-preexec-fn
    with subprocess.Popen(
        command,
        shell=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        preexec_fn=os.setsid,
    ) as process:
        try:
            logging.debug("Waiting on process for %ss", timeout)
            start_time = time.time()
            process.wait(timeout=timeout)
            logging.debug("Took %ss", time.time() - start_time)
            assert process.stdout
            assert process.stderr
            return process.stdout.read(), process.stderr.read()
        except subprocess.TimeoutExpired as e:
            assert process.stdout
            assert process.stderr
            stdout = process.stdout.read()
            stderr = process.stderr.read()
            logging.debug("%s, %s, %s", e, stdout, stderr)
            os.killpg(os.getpgid(process.pid), signal.SIGTERM)
            return stdout, stderr


def run_with_fifos(
    top_level_module: str,
    input_fifos: dict[str, str],
    timeout: Optional[int] = None,
):
    """
    Run iverilog with fifos

    :return: (stdout, stderr/exception)
    """
    iverilog_cmd = make_cmd(
        top_level_module=top_level_module,
        files=input_fifos.keys(),
    )
    logging.debug("Simulation with command:\n%s", iverilog_cmd)
    return _run_cmd_with_fifos(
        command=iverilog_cmd, input_fifos=input_fifos, timeout=timeout
    )


def run_with_files(
    top_level_module: str,
    input_files: dict[str, str],
    timeout: Optional[int] = None,
):
    """
    Run iverilog with files

    :return: (stdout, stderr/exception)
    """
    iverilog_cmd = make_cmd(
        top_level_module=top_level_module,
        files=input_files.keys(),
    )
    logging.info("Made command `%s`", iverilog_cmd)
    return _run_cmd_with_files(
        command=iverilog_cmd, input_files=input_files, timeout=timeout
    )
