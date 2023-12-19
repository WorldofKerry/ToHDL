module hrange_tb (
);
    reg _clock;
    reg _start;
    reg _reset;
    reg _ready;
    reg signed [31:0] base;
    reg signed [31:0] limit;
    reg signed [31:0] step;
    wire _done;
    wire _valid;
    wire signed [31:0] _0;
    wire signed [31:0] _1;
    hrange DUT (
        ._clock(_clock),
        ._start(_start),
        ._reset(_reset),
        ._ready(_ready),
        .base(base),
        .limit(limit),
        .step(step),
        ._done(_done),
        ._valid(_valid),
        ._0(_0),
        ._1(_1)
        );
    always #5 _clock = !_clock;
    initial begin
        _clock = 0;
        _start = 0;
        _ready = 1;
        _reset = 1;
        @(negedge _clock);
        _reset = 0;
        // ============ Test Case 0 with arguments (0, 10, 2) ============
        base = $signed(0);
        limit = $signed(10);
        step = $signed(2);
        _start = 1;
        @(negedge _clock);
        base = 'x; // only need inputs when start is set
        limit = 'x; // only need inputs when start is set
        step = 'x; // only need inputs when start is set
        _start = 0;
        while ((!(_done) || !(_ready))) begin
            // `if (_ready && _valid)` also works as a conditional
            if (_ready) begin
                $display("%0d, %0d, %0d, %0d", _valid, _ready, _0, _1);
            end
            @(negedge _clock);
        end
        if (_ready) begin
            $display("%0d, %0d, %0d, %0d", _valid, _ready, _0, _1);
        end
        // ============ Test Case 1 with arguments (0, 10, 2) ============
        base = $signed(0);
        limit = $signed(10);
        step = $signed(2);
        _start = 1;
        @(negedge _clock);
        base = 'x; // only need inputs when start is set
        limit = 'x; // only need inputs when start is set
        step = 'x; // only need inputs when start is set
        _start = 0;
        while ((!(_done) || !(_ready))) begin
            // `if (_ready && _valid)` also works as a conditional
            if (_ready) begin
                $display("%0d, %0d, %0d, %0d", _valid, _ready, _0, _1);
            end
            @(negedge _clock);
        end
        if (_ready) begin
            $display("%0d, %0d, %0d, %0d", _valid, _ready, _0, _1);
        end
        // ============ Test Case 2 with arguments (0, 10, 2) ============
        base = $signed(0);
        limit = $signed(10);
        step = $signed(2);
        _start = 1;
        @(negedge _clock);
        base = 'x; // only need inputs when start is set
        limit = 'x; // only need inputs when start is set
        step = 'x; // only need inputs when start is set
        _start = 0;
        while ((!(_done) || !(_ready))) begin
            // `if (_ready && _valid)` also works as a conditional
            if (_ready) begin
                $display("%0d, %0d, %0d, %0d", _valid, _ready, _0, _1);
            end
            @(negedge _clock);
        end
        if (_ready) begin
            $display("%0d, %0d, %0d, %0d", _valid, _ready, _0, _1);
        end
        $finish;
    end
endmodule
module dup_range_goal_tb (
);
    reg _clock;
    reg _start;
    reg _reset;
    reg _ready;
    reg signed [31:0] base;
    reg signed [31:0] limit;
    reg signed [31:0] step;
    wire _done;
    wire _valid;
    wire signed [31:0] _0;
    dup_range_goal DUT (
        ._clock(_clock),
        ._start(_start),
        ._reset(_reset),
        ._ready(_ready),
        .base(base),
        .limit(limit),
        .step(step),
        ._done(_done),
        ._valid(_valid),
        ._0(_0)
        );
    always #5 _clock = !_clock;
    initial begin
        _clock = 0;
        _start = 0;
        _ready = 1;
        _reset = 1;
        @(negedge _clock);
        _reset = 0;
        // ============ Test Case 0 with arguments (0, 10, 2) ============
        base = $signed(0);
        limit = $signed(10);
        step = $signed(2);
        _start = 1;
        @(negedge _clock);
        base = 'x; // only need inputs when start is set
        limit = 'x; // only need inputs when start is set
        step = 'x; // only need inputs when start is set
        _start = 0;
        while ((!(_done) || !(_ready))) begin
            // `if (_ready && _valid)` also works as a conditional
            if (_ready) begin
                $display("%0d, %0d, %0d", _valid, _ready, _0);
            end
            @(negedge _clock);
        end
        if (_ready) begin
            $display("%0d, %0d, %0d", _valid, _ready, _0);
        end
        $finish;
    end
endmodule
