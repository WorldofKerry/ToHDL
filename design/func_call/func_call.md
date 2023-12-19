# Function Calls

## Abstract

Function calls are a crutial part of programming languages.

How can this abstraction be converted to hardware?

## Example

Let there be a function `hrange`, which is a more basic implementation of the built-in `range` function. This function is already [convertible by the tool](./hrange.sv).

```python
@verilogify(mode=Modes.OVERWRITE)
def hrange(base, limit, step):
    i = base
    while i < limit:
        yield i
        i += step
```

Lets make another function called `dup_range` that calls `hrange`, and uses its outputs in its own yields.

```python
@verilogify(mode=Modes.OVERWRITE)
def dup_range(base, limit, step):
    inst = hrange(base, limit, step) # 1
    for value in inst: # 2
        yield value # 3
        yield value + 1 # 4
```

How can we convert the function all portion of this into Verilog?

```verilog
module dup_range(
    // ...
);
// ...
    hrange _inst(
        .base(_hrange_inst_base),
        .limit(_hrange_inst_limit),
        .step(_hrange_inst_step),
        ._0(_hrange_inst__0),
        ._ready(_hrange_inst__ready),
        // ...
    )
    // ...
    case (_state) begin
        _state_1: begin // instantiation node
            _hrange_inst_base <= _base;
            _hrange_inst_limit <= _limit;
            _hrange_inst_base <= _step;
            _hrange_inst__start <= 1;
            _hrange_inst__ready <= 0;
            _state <= _state_2;
        end
        _state_2: begin // iterate node
            _hrange_inst__ready <= 1;
            if (_hrange_inst__ready && _hrange_inst__valid) begin
                _value <= _hrange_inst__0;
                _state <= _state_3;
                _hrange_inst__ready <= 0;
            end
            if (_hrange_inst__done) begin
                _state <= _state_done;
                _hrange_inst__ready <= 0;
            end
        end
        _state_3: begin
            valid <= 1;
            _0 <= value;
            _state <= _state_4;
        end
        _state_4: begin
            valid <= 1;
            _0 <= value + 1;
            _state <= _state_2;
        end
        _state_done: // ...
        // ...
    endmodule
```

## Proposal

Two new nodes will need to be added to the intermediate representation, to represent instantiating and iterating a generator respectively.

A new variable type will have to be added for generator instances

### Instantiation Node

Represents the instantiation of a generator in Python, e.g. `inst = hrange(a, b, c)`.
Represents a module that has been "started" in Verilog.

Also encapsulates the arguments passed in by the callee.

Members:

- Instance name
- Arguments Passed
- Callee function/module name

### Iterate Node

For a instance `inst` of generator `hrange`, and caller variables `a` and `b`, this node represents a

- `a, b = next(inst)` call on a generator instance, or a
- `for a, b in inst`, or a
- portion of a `yield from inst`

Members:

- Instance name, e.g. `inst`
- Output mapping, e.g. in the above example `a` maps to [`_0`](./hrange.sv) of `hrange`

### Generator Instance Variable

Abstraction for a variable that holds a generator instance.
Represented by a module in Verilog.

Members:

- name
- python function "pointer", for key in the namespace

## Timeline

### Milestone 1 - Concreteness

1. Finish Verilog pseudocode and verify - 1 point
1. Draw intermdiate representation for `dup_range` - 1 point

### Milestone 2 - Preparation

1. Update tests to support multiple functions in one file - 2 points
1. Reword `Var` class to handle different types (or allow for inheritance) - 3 points

### Milestone 3 - Function Calls

1. (Optional) improve graph visualzation tooling if debugging becomes inefficient - 2 points
1. Add the two new nodes and generator instance variable - 3 points
   1. including handling it in the Verilog codegen
1. Add function parse ability to frontend - 2 points
   1. start with only supporting `inst = gen(a, b, c); for x, y in inst: ...`
   1. (Optional) add support for `yield from` and `for x, y in gen(a, b, c)`

## Finish Verilog pseudocode and verify

```bash
# manual
iverilog -s dup_range_tb ./design/func_call/dup_range.sv ./design/func_call/dup_range_tb.sv ./design/func_call/hrange.sv -g2005-sv -o output.log && vvp output.log

# goal
iverilog -s dup_range_goal_tb ./design/func_call/dup_range_goal.sv ./design/func_call/dup_range_goal_tb.sv ./design/func_call/hrange.sv -g2005-sv -o output.log && vvp output.log

# triple circle
iverilog -s triple_circle_tb tests/api/triple_circle.sv tests/api/triple_circle_tb.sv  -g2005-sv -Wall && vvp a.out
```
