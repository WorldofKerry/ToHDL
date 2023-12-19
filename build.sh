#! /bin/bash -ex

dev() {
    pip install -e .[dev]
}

venv() {
    if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
        echo "Run this command as source"
        exit 1
    else
        python3 -m venv venv
        source venv/bin/activate

        dev

        # Re-activate for pytest
        deactivate && source venv/bin/activate
    fi
}

rust() {
    cd crates/pytohdl/ && pip install .
}

ci() {
    rust
    dev
}

"$@"
