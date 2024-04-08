#! /bin/bash -ex

venv() {
    if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
        echo "Run this command as source"
        exit 1
    else
        python3 -m venv .venv
        source .venv/bin/activate

        dev

        # Re-activate for pytest
        deactivate && source .venv/bin/activate
    fi
}

# Installs pytohdl as package for Python Interpretor
rust() {
    cd crates/pytohdl/ && pip install .
    cd ../../
}

# Installs python2verilog's dev dependencies
dev() {
    pip install -e .[dev]
}

all() {
    rust
    dev
}

"$@"
