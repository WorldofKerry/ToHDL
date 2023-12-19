# Random Planning, Design, and Notes

## APIs

### Filesystem -> CLI -> Filesystem

`python3 -m python2verilog ./fib.py` outputs to `./fib_module.sv` and `./fib_testbench.sv`

### Text -> Func -> Text

```python
from python2verilog import convert
code = """
def fib(n):
  yield(n)
fib(10)
"""
print(convert("fib", code))
```

### File -> Func -> File

```python
from python2verilog import convert
def fib(n):
  yield(n)
fib(10)
print(convert("fib")) # Using this file as input
```

## Older

### More Variable Data

How to convert a Python `ast.Name`? We want to be able to map those variables to

### Start, Ready, Reset

How should these signals be handled while minimizing unnecessary clock cycles?

- If module sees high `reset`, module resets and holds `ready` high one cycle later
- If module sees high `start`, module captures inputs in the same cycle, then holds `ready` low until done
- If user sees high `ready`, they can assert `start` in the same clock cycle (e.g. connect `ready` to `start` is valid)

### What needs to be duplicated in testbenches?

declare I/O and other signals
declare DUT
start clock

loop for each test case

- start signal
- while wait for done signal
  - clock
  - set start zero
  - display output
    endloop

### Potential API

```python
import python2verilog as p2v
import ast

func = ast.parse(code).body[0]

ir = p2v.from_python_get_ir(func.body)

# Optimization passes
ir = p2v.optimizations.replace_single_item_cases(ir)
ir = p2v.optimizations.remove_nesting(ir)
ir = p2v.optimizations.combine_statements(ir)

verilog = p2v.Verilog()
verilog.from_python_do_setup(ir.get_context()) # module I/O is dependent on Python
verilog.from_ir_fill_body(ir.get_root()) # fills the body

# whether has valid or done signal,
# whether initialization is always happening or only on start,
# verilog sim name
verilog.config(has_valid=True, has_done=True, lazy_start=True, verilog_sim="iverilog")

with open(f"{verilog.get_module_name()}.sv", mode="w") as module:
  module.write(verilog.get_module())
with open(f"{verilog.get_module_name()}_tb.sv", mode="w") as tb:
  tb.write(verilog.get_testbench())

print(verilog.get_verilog_run_cmd())
assert verilog.test_outputs() # checks if verilog and python output same
```

### Rectangle Filled

```python
def draw_rectangle(s_x, s_y, height, width) -> tuple[int, int]:
    for i0 in range(0, width):
        for i1 in range(0, height):
            yield (s_x + i1, s_y + i0)
```

```verilog
case (STATE)
  0: begin
    if (i0 < width) begin
      STATE <= STATE_1;
    end else begin
      case (STATE_INNER)
        0: begin
          if (i1 < height) begin
            STATE_INNER <= STATE_INNER + 1;
          end else begin
            case (STATE_INNER_INNER)
              0: begin
                out0 <= s_x + i1;
                out1 <= s_y + i0;
                i1 <= i1 + 1;

                STATE_INNER_INNER <= STATE_INNER_INNER + 1;
                STATE_INNER_INNER <= 0; // flag to either wrap around or remain
              end
          end
          STATE_INNER <= 0;
        end
      endcase
    end
  end
  STATE_1: begin
    done <= 1;
  end
endcase
```

## Converting a While Loop

```python
i = 0
while <condition>:
    <statement 1>
    <statement 2>
    ...
```

```verilog
case (STATE)
  0: begin
    // For loop start
    if (condition) begin
      STATE <= STATE + 1;
    end else begin
      case (STATE_INNER)
        0: begin
          // statement 1
        end
        1: begin
          // statement 2
        end
        // ...
        10: begin
          STATE_INNER <= 0;
        end
      endcase
    end
    // For loop end
  end
  // ...
endcase
```

## If Statement Analysis

```verilog
// IF START
case (_STATE_IF)
  0: begin
    if (condition) _STATE_IF <= 1;
    else _STATE_IF <= 2;
  end
  1: begin
    // THEN BODY START
    case ()
    // ...
      _STATE_IF <= 0;
    // THEN BODY END
  end
  2: begin
    // ELSE BODY START
    // ...
     __STATE_IF <= 0;
    // ELSE BODY END
  end
endcase
// IF END
```
