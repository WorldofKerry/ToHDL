# State Machine as a Use Case

Most state machines also need inputs at each clock cycle.

This can be achieved using the `.send` method of a generator instance.

This will also require design on parsing `yield` statements that return values, e.g. `input = yield output`.
