module even_fib (
    // Function parameters (only need to be set when start is high):
    input wire signed [31:0] n,

    input wire __clock, // clock for sync
    input wire __reset, // set high to reset, i.e. done will be high
    input wire __start, // set high to capture inputs (in same cycle) and start generating

    // Implements a ready/valid handshake based on
    // http://www.cjdrake.com/readyvalid-protocol-primer.html
    input wire __ready, // set high when caller is ready for output
    output reg __valid, // is high if output values are valid

    output reg __done, // is high if module done outputting

    // Output values as a tuple with respective index(es)
    output reg signed [31:0] __output_0
);
    // State variables
    localparam _state0_assign0 = 0;
    localparam _state3_while = 1;
    localparam _state3_while0 = 2;
    localparam _state3_while1_assign0 = 3;
    localparam _state3_while2_assign0 = 4;
    localparam _state3_while3_assign0 = 5;
    localparam _state3_while4_assign0 = 6;
    localparam _state4_assign0 = 7;
    localparam _state_done = 8;
    localparam _state_idle = 9;
    reg [31:0] _state;
    // Local variables
    reg signed [31:0] _a;
    reg signed [31:0] _b;
    reg signed [31:0] _i;
    reg signed [31:0] _temp;
    reg signed [31:0] _n;
    // Core
    always @(posedge __clock) begin
        `ifdef DEBUG
        $display("even_fib,%s,__start=%0d,__done=%0d,__ready=%0d,__valid=%0d,n=%0d,_n=%0d,__output_0=%0d,_i=%0d,_a=%0d,_b=%0d,_temp=%0d", _state.name, __start, __done, __ready, __valid, n, _n, __output_0, _i, _a, _b, _temp);
        `endif
        if (__ready) begin
            __valid <= 0;
            __done <= 0;
        end
        // Start signal takes precedence over reset
        if ((__reset || __start)) begin
            _state <= _state_idle;
            __done <= 0;
            __valid <= 0;
        end
        if (__start) begin
            _n <= n;
            _i <= $signed(0);
            _a <= $signed(0);
            _b <= $signed(1);
            if (($signed(0) < n)) begin
                if ($signed($signed($signed($signed(0) % $signed(2)) + $signed(2)) % $signed(2))) begin
                    __output_0 <= $signed(0);
                    __valid <= 1;
                    _temp <= $signed($signed(0) + $signed(1));
                    _a <= $signed(1);
                    _b <= $signed($signed(0) + $signed(1));
                    _i <= $signed($signed(0) + $signed(1));
                    _state <= _state3_while;
                end else begin
                    _temp <= $signed($signed(0) + $signed(1));
                    _a <= $signed(1);
                    _b <= $signed($signed(0) + $signed(1));
                    _i <= $signed($signed(0) + $signed(1));
                    _state <= _state3_while;
                end
            end else begin
                __done <= 1;
                __valid <= 1;
                _state <= _state_idle;
            end
        end else begin
            // If ready or not valid, then continue computation
            if ((__ready || !(__valid))) begin
                case (_state)
                    _state3_while: begin
                        if ((_a < _n)) begin
                            if ($signed($signed($signed(_a % $signed(2)) + $signed(2)) % $signed(2))) begin
                                __output_0 <= _a;
                                __valid <= 1;
                                _temp <= $signed(_a + _b);
                                _a <= _b;
                                _b <= $signed(_a + _b);
                                _i <= $signed(_i + $signed(1));
                                if ((_b < _n)) begin
                                    _state <= _state3_while0;
                                end else begin
                                    _state <= _state4_assign0;
                                end
                            end else begin
                                _temp <= $signed(_a + _b);
                                _a <= _b;
                                _b <= $signed(_a + _b);
                                _i <= $signed(_i + $signed(1));
                                if ((_b < _n)) begin
                                    _state <= _state3_while0;
                                end else begin
                                    __done <= 1;
                                    __valid <= 1;
                                    _state <= _state_idle;
                                end
                            end
                        end else begin
                            __done <= 1;
                            __valid <= 1;
                            _state <= _state_idle;
                        end
                    end
                    _state3_while0: begin
                        if ($signed($signed($signed(_a % $signed(2)) + $signed(2)) % $signed(2))) begin
                            __output_0 <= _a;
                            __valid <= 1;
                            _temp <= $signed(_a + _b);
                            _a <= _b;
                            _b <= $signed(_a + _b);
                            _i <= $signed(_i + $signed(1));
                            if ((_b < _n)) begin
                                _state <= _state3_while0;
                            end else begin
                                _state <= _state4_assign0;
                            end
                        end else begin
                            _temp <= $signed(_a + _b);
                            _a <= _b;
                            _b <= $signed(_a + _b);
                            _i <= $signed(_i + $signed(1));
                            if ((_b < _n)) begin
                                if ($signed($signed($signed(_b % $signed(2)) + $signed(2)) % $signed(2))) begin
                                    __output_0 <= _b;
                                    __valid <= 1;
                                    _state <= _state3_while1_assign0;
                                end else begin
                                    _temp <= $signed(_a + _b);
                                    _a <= _b;
                                    _b <= $signed(_a + _b);
                                    _i <= $signed(_i + $signed(1));
                                    if ((_b < _n)) begin
                                        if ($signed($signed($signed(_b % $signed(2)) + $signed(2)) % $signed(2))) begin
                                            __output_0 <= _b;
                                            __valid <= 1;
                                            _temp <= $signed(_b + $signed(_a + _b));
                                            _state <= _state3_while2_assign0;
                                        end else begin
                                            _temp <= $signed(_b + $signed(_a + _b));
                                            _state <= _state3_while2_assign0;
                                        end
                                    end else begin
                                        __done <= 1;
                                        __valid <= 1;
                                        _state <= _state_idle;
                                    end
                                end
                            end else begin
                                __done <= 1;
                                __valid <= 1;
                                _state <= _state_idle;
                            end
                        end
                    end
                    _state3_while1_assign0: begin
                        _temp <= $signed(_a + _b);
                        _a <= _b;
                        _b <= $signed(_a + _b);
                        _i <= $signed(_i + $signed(1));
                        if ((_b < _n)) begin
                            if ($signed($signed($signed(_b % $signed(2)) + $signed(2)) % $signed(2))) begin
                                __output_0 <= _b;
                                __valid <= 1;
                                _temp <= $signed(_b + $signed(_a + _b));
                                _state <= _state3_while2_assign0;
                            end else begin
                                _temp <= $signed(_b + $signed(_a + _b));
                                _state <= _state3_while2_assign0;
                            end
                        end else begin
                            __done <= 1;
                            __valid <= 1;
                            _state <= _state_idle;
                        end
                    end
                    _state3_while2_assign0: begin
                        _a <= _b;
                        _b <= _temp;
                        _i <= $signed(_i + $signed(1));
                        if ((_b < _n)) begin
                            if ($signed($signed($signed(_b % $signed(2)) + $signed(2)) % $signed(2))) begin
                                __output_0 <= _b;
                                __valid <= 1;
                                _temp <= $signed(_b + _temp);
                                _a <= _temp;
                                _state <= _state3_while3_assign0;
                            end else begin
                                _temp <= $signed(_b + _temp);
                                _a <= _temp;
                                _state <= _state3_while3_assign0;
                            end
                        end else begin
                            __done <= 1;
                            __valid <= 1;
                            _state <= _state_idle;
                        end
                    end
                    _state3_while3_assign0: begin
                        _b <= _temp;
                        _i <= $signed(_i + $signed(1));
                        if ((_a < _n)) begin
                            if ($signed($signed($signed(_a % $signed(2)) + $signed(2)) % $signed(2))) begin
                                __output_0 <= _a;
                                __valid <= 1;
                                _temp <= $signed(_a + _temp);
                                _a <= _temp;
                                _b <= $signed(_a + _temp);
                                _state <= _state3_while4_assign0;
                            end else begin
                                _temp <= $signed(_a + _temp);
                                _a <= _temp;
                                _b <= $signed(_a + _temp);
                                _state <= _state3_while4_assign0;
                            end
                        end else begin
                            __done <= 1;
                            __valid <= 1;
                            _state <= _state_idle;
                        end
                    end
                    _state3_while4_assign0: begin
                        _i <= $signed(_i + $signed(1));
                        if ((_a < _n)) begin
                            if ($signed($signed($signed(_a % $signed(2)) + $signed(2)) % $signed(2))) begin
                                __output_0 <= _a;
                                __valid <= 1;
                                _temp <= $signed(_a + _b);
                                _a <= _b;
                                _b <= $signed(_a + _b);
                                _i <= $signed($signed(_i + $signed(1)) + $signed(1));
                                _state <= _state3_while;
                            end else begin
                                _temp <= $signed(_a + _b);
                                _a <= _b;
                                _b <= $signed(_a + _b);
                                _i <= $signed($signed(_i + $signed(1)) + $signed(1));
                                _state <= _state3_while;
                            end
                        end else begin
                            __done <= 1;
                            __valid <= 1;
                            _state <= _state_idle;
                        end
                    end
                    _state4_assign0: begin
                        __done <= 1;
                        __valid <= 1;
                        _state <= _state_idle;
                    end
                endcase
            end
        end
    end
endmodule

