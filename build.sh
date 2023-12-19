#! /bin/bash -ex

dep() {
    pip install -e .[dev]
}

venv() {
    if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
        echo "Run this command as source"
        exit 1
    else
        python3 -m venv venv
        source venv/bin/activate

        dep

        # Re-activate for pytest
        deactivate && source venv/bin/activate
    fi
}

rust() {
    # maturin develop --manifest-path crates/pytohdl/Cargo.toml
    cd crates/pytohdl/ && pip install .
}

ci() {
    dep
    rust
}

"$@"
