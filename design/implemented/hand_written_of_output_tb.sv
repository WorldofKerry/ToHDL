module testing_tb (
);
    reg _clock;
    reg _start;
    reg signed [31:0] n;
    wire signed [31:0] _out0;
    wire _done;
    wire _valid;
    testing DUT (
        ._clock(_clock),
        ._start(_start),
        .n(n),
        ._out0(_out0),
        ._done(_done),
        ._valid(_valid)
        );
    always #5 _clock = !_clock;
    initial begin
        _clock = 0;
        _start = 0;
        @(negedge _clock);

        // Test case 0: (15,)
        n = 15;
        _start = 1;
        @(negedge _clock);
        while (!_done) begin
            _start = 0;
            $display("%0d, %0d", _valid, _out0);
            @(negedge _clock);
        end
        $display("%0d, %0d", _valid, _out0);

        $finish;
    end
endmodule
