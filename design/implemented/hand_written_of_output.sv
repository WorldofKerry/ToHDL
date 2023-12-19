module testing (
    input wire [31:0] n,
    input wire _start,
    input wire _clock,
    output reg [31:0] _out0,
    output reg _done,
    output reg _valid
);
    reg signed [31:0] i;
    reg signed [31:0] _1_while_0_t_0;
    reg signed [31:0] _1_while_0_f_0;
    reg signed [31:0] _1_while_0;
    reg signed [31:0] _statelmaodone;
    reg signed [31:0] _0;
    reg signed [31:0] _1_while;
    reg signed [31:0] _2;
    reg signed [31:0] _state;
    always @(posedge _clock) begin
        _valid <= 0;
        _done <= 0;
        if (_start) begin
            _done <= 0;
            i <= 0;
            _1_while_0_t_0 <= 1;
            _1_while_0_f_0 <= 2;
            _1_while_0 <= 3;
            _statelmaodone <= 4;
            _0 <= 5;
            _1_while <= 6;
            _2 <= 7;
            _state <= 7;
        end else begin
            case (_state)
              	_1_while: begin
                  if (i < n) begin
                    if (i > 1) begin
                      i <= (i + 1);
                    end else begin
                      i <= (i + 2);
                    end
                  end else begin
                    _out0 <= i;
					_state <= _statelmaodone;
                  end
                end
                _2: begin
                  i <= 0;
                  if (0 < n) begin
                    if (0 > 1) begin
                      i <= 0 + 1;
                   	  _state <= _1_while;
                    end else begin
                      i <= 0 + 2;
                   	  _state <= _1_while;
                    end
                  end else begin
                    _out0 <= 0;
                    _state <= _statelmaodone;
                  end
                end
                _statelmaodone: begin
                    _done <= 1;
                end
            endcase
        end
    end
endmodule
