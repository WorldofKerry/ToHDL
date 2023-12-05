# ToHDL

## Flamegraph

```bash
sudo apt install linux-tools-common linux-tools-generic linux-tools-`uname -r`

sudo sysctl kernel.perf_event_paranoid=0

cargo flamegraph --unit-test tohdl-codegen -- verilog::module::test::odd_fib
```
