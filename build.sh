#! /bin/bash -ex
setup_venv() {
    python3 -m venv venv
    source venv/bin/activate
    python3 -m pip install -e .[dev]

    # Re-activate for pytest
    deactivate && source venv/bin/activate
}

build_rust() {
    maturin develop --manifest-path crates/pytohdl/Cargo.toml
}

if [[ "$VIRTUAL_ENV" == "" ]]; then
    if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
        echo "Run this command as source"
        exit 1
    else
        setup_venv
    fi
fi

build_rust