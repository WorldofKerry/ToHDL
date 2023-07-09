module draw_rectangle(
    input wire _clock,
    input wire _start,
    input wire [31:0] s_x,
    input wire [31:0] s_y,
    input wire [31:0] height,
    input wire [31:0] width,
    output reg [31:0] _out0,
    output reg [31:0] _out1,
    output reg _done
);
    reg [31:0] _STATE;
    reg [31:0] _STATE_0;
    reg [31:0] i0;
    reg [31:0] _INNER_1_STATE;
    reg [31:0] _INNER_1_STATE_0;
    reg [31:0] _INNER_1_STATE_1;
    reg [31:0] _INNER_1_STATE_2;
    reg [31:0] _STATE_1;
    reg [31:0] i1;
    reg [31:0] _INNER_2_STATE;
    reg [31:0] _INNER_2_STATE_0;
    reg [31:0] _INNER_2_STATE_1;
    reg [31:0] _INNER_2_STATE_2;
    reg [31:0] _STATE_2;
        always @(posedge _clock) begin
            if (_start) begin
                _done <= 0;
                _STATE <= 0;
                _STATE_0 <= 0;
                i0 <= 0;
                _INNER_1_STATE <= 0;
                _INNER_1_STATE_0 <= 0;
                _INNER_1_STATE_1 <= 1;
                _INNER_1_STATE_2 <= 2;
                _STATE_1 <= 1;
                i1 <= 0;
                _INNER_2_STATE <= 0;
                _INNER_2_STATE_0 <= 0;
                _INNER_2_STATE_1 <= 1;
                _INNER_2_STATE_2 <= 2;
                _STATE_2 <= 2;
            end else begin
                case (_STATE) // STATEMENTS START
                    _STATE_0: begin
                        if (i0 >= width) begin // FOR LOOP START
                            _STATE = _STATE + 1;
                            i0 <= 0;
                        end else begin // FOR LOOP BODY
                            case (_INNER_1_STATE) // STATEMENTS START
                                _INNER_1_STATE_0: begin
                                    _out0 <= s_x;
                                    _out1 <= s_y + i0;
                                    _INNER_1_STATE <= _INNER_1_STATE + 1;
                                end
                                _INNER_1_STATE_1: begin
                                    _out0 <= s_x + height - 1;
                                    _out1 <= s_y + i0;
                                    _INNER_1_STATE <= _INNER_1_STATE + 1;
                                end
                                _INNER_1_STATE_2: begin // END STATEMENTS STATE
                                    i0 <= i0 + 1;
                                    _INNER_1_STATE <= 0; // LOOP FOR LOOP STATEMENTS
                                end
                            endcase // STATEMENTS END
                        end // FOR LOOP END
                    end
                    _STATE_1: begin
                        if (i1 >= height) begin // FOR LOOP START
                            _STATE = _STATE + 1;
                            i1 <= 0;
                        end else begin // FOR LOOP BODY
                            case (_INNER_2_STATE) // STATEMENTS START
                                _INNER_2_STATE_0: begin
                                    _out0 <= s_x + i1;
                                    _out1 <= s_y;
                                    _INNER_2_STATE <= _INNER_2_STATE + 1;
                                end
                                _INNER_2_STATE_1: begin
                                    _out0 <= s_x + i1;
                                    _out1 <= s_y + width - 1;
                                    _INNER_2_STATE <= _INNER_2_STATE + 1;
                                end
                                _INNER_2_STATE_2: begin // END STATEMENTS STATE
                                    i1 <= i1 + 1;
                                    _INNER_2_STATE <= 0; // LOOP FOR LOOP STATEMENTS
                                end
                            endcase // STATEMENTS END
                        end // FOR LOOP END
                    end
                    _STATE_2: begin // END STATEMENTS STATE
                        _done = 1;
                    end
                endcase // STATEMENTS END
            end
        end
endmodule
