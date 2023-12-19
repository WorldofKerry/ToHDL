module hrange (
    input wire signed [31:0] base,
    input wire signed [31:0] limit,
    input wire signed [31:0] step,
    input wire _clock,
    input wire _reset,
    input wire _start, // set high to capture inputs (in same cycle) and start generating
    input wire _wait, // set high to have module pause outputting
    output reg _ready, // is high if module done outputting
    output reg _valid, // is high if output is valid
    output reg signed [31:0] _0
);
    localparam _state_1 = 0;
    localparam _state_0_while_0 = 1;
    localparam _statelmaoready = 2;
    reg signed [31:0] _i;
    reg signed [31:0] _state;
    reg signed [31:0] _base;
    reg signed [31:0] _limit;
    reg signed [31:0] _step;
    always @(posedge _clock) begin
        _valid <= 0;
        _ready <= 0;
        _0 <= $signed(0);

        // Start signal takes precedence over reset
        if (_reset) begin
            _state <= _statelmaoready;
        end

        // if (_valid && _wait)
        //     continue;

        if (_start) begin
            _base <= base;
            _limit <= limit;
            _step <= step;
            if (_wait) begin
                // delayed start
                _state = _state_delayed_start;
            end else begin
                // quick start
                _i <= base;
                if ((base < limit)) begin
                    _0 <= base;
                    _valid <= 1;
                    _state <= _state_0_while_0;
                end else begin
                    _ready <= 1;
                    _state <= _statelmaoready;
                end
            end
        end else begin
            if (!(_wait)) begin
                case (_state)
                    _state_delayed_start: begin
                        _i <= _base;
                        if ((_base < limit)) begin
                            _0 <= _base;
                            _valid <= 1;
                            _state <= _state_0_while_0;
                        end else begin
                            _ready <= 1;
                            _state <= _statelmaoready;
                        end
                    end
                    _state_0_while_0: begin
                        _i <= $signed(_i + _step);
                        if (($signed(_i + _step) < _limit)) begin
                            _0 <= $signed(_i + _step);
                            _valid <= 1;
                            _state <= _state_0_while_0;
                        end else begin
                            _ready <= 1;
                            _state <= _statelmaoready;
                        end
                    end
                endcase
            end
        end
    end
endmodule
