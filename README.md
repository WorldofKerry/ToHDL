# ToHDL

Infrastructure for generating fsm-like code.

Currently supports:

- Python frontend and backend
- Verilog backend

## IVerilog

```bash
clear && iverilog -g2005-sv -Wall ./p2v.sv ./p2v_tb.sv && unbuffer vvp a.out
```

## Flamegraph

```bash
sudo apt install linux-tools-common linux-tools-generic linux-tools-`uname -r`

sudo sysctl kernel.perf_event_paranoid=0

CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --unit-test tohdl-codegen -- verilog::module::test::odd_fib
```
