[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/WorldofKerry/Python2Verilog/blob/main/examples/notebook.ipynb)
[![pypi](https://img.shields.io/pypi/v/python2verilog?label=pypi%20package)](https://pypi.org/project/python2verilog/)
![py versions](https://img.shields.io/badge/dynamic/yaml?url=https%3A%2F%2Fraw.githubusercontent.com%2FWorldofKerry%2FPython2Verilog%2Fmain%2F.github%2Fworkflows%2Fpytest.yml&query=%24.jobs.full.strategy.matrix%5B%22python-version%22%5D&label=python%20versions)
[![pypi downloads](https://img.shields.io/pypi/dm/python2verilog)](https://pypi.org/project/python2verilog/)
[![pytest](https://github.com/worldofkerry/python2verilog/actions/workflows/pytest.yml/badge.svg)](https://github.com/WorldofKerry/Python2Verilog/actions/workflows/pytest.yml)

# python2verilog

- This tool facilitates the conversion of select Python functions (including generators!) into synthesizable sequential SystemVerilog
- Ideal for quickly translating higher-level "CPU code" into hardware descriptions for use on FPGAs, without needing to interface with or including a CPU in the design
- Testbenches can be automatically generated if the user uses the function within their Python code or provides explicit test cases

```python
from python2verilog import verilogify

@verilogify
def hrange(base, limit, step):
    i = base
    while i < limit:
        yield i
        i += step
print(list(hrange(0, 10, 3)))
```
A live transpile demo can be found [here](https://python2verilog-live.vercel.app/).

## Specifications

Some constrains on Python functions include:

- Supports only signed integral input/output and operations
- Must be a [pure function](https://en.wikipedia.org/wiki/Pure_function)

Unsupported Python paradigms include but are not limited to the following:

- Global (nonlocal) variables, instead declare them within the function with minimal overhead
- Keyword parameters and default arguments, instead use explicit positional arguments

## Usage and Installation

Try it in [Google Collab](https://colab.research.google.com/github/WorldofKerry/Python2Verilog/blob/main/examples/notebook.ipynb) or check out [`examples/`](examples/)!

`python3 -m pip install --upgrade pip`

`python3 -m pip install python2verilog`

## Tested Generations

You may find the output of the [integration testing](tests/integration/functions.py) as a [Github Artifact](https://nightly.link/WorldofKerry/Python2Verilog/workflows/pytest/main/tests-data.zip) available for download.

## For Developers

To setup pre-commit, run `pre-commit install`.

[Github Issues](https://github.com/WorldofKerry/Python2Verilog/issues) are used for tracking.

Sphinx is used for the docs. Follow the [sphinx workflow](.github/workflows/sphinx.yml) to generate a local copy.

## Development

### Setup

For most up-to-date information, refer to the [pytest workflow](.github/workflows/pytest.yml) or the [packaging workflow](.github/workflows/packaging.yml).

A Ubuntu environment (WSL2 works too, make sure to have the repo on the Ubuntu partition, as [`os.mkfifo`](https://docs.python.org/3/library/os.html#os.mkfifo) is used to avoid writing to disk)

Steps
```bash
source ./build.sh venv
./build.sh all
pre-commit install
```

For automatic Verilog simulation and testing, install [Icarus Verilog](https://github.com/steveicarus/iverilog) and its dependencies with
`sudo apt-get install iverilog expected` (uses the `unbuffer` in `expected`).

The online simulator [EDA Playground](https://edaplayground.com/) can be used as a subsitute if you manually copy-paste the module and testbench files to it.

### Running Tests

Run Python tests with `pytest`.
Run rust tests with `cargo test`.

CLI arguments for test configuration can be found in [tests/conftest.py](tests/conftest.py).

Use `./clean.sh` to remove gitignored and generated files.

### Troubleshooting

#### `cargo test` Errors
- error: linker \`cc\` not found
    - On Ubuntu run `sudo apt install build-essential`
- /usr/bin/ld: cannot find -lpython3.10: No such file or directory
    - On Ubuntu 22.04 run `sudo apt install libpython3.10-dev`

## Flamegraph

```bash
sudo apt install linux-tools-common linux-tools-generic linux-tools-`uname -r`

sudo sysctl kernel.perf_event_paranoid=0

CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --unit-test tohdl-codegen -- verilog::module::test::odd_fib
```
