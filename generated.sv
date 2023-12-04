/*
iverilog '-Wall' '-g2012' generated.sv  && unbuffer vvp a.out
*/
module fib (
    input logic [31:0] n,
    input logic ready,
    input logic start,
    input logic clock,
    input logic reset,
    output logic valid,
    output logic done,
    output logic [31:0] out_0
);
    logic [31:0] mem_0;
    logic [31:0] mem_1;
    logic [31:0] mem_2;
    logic [31:0] mem_3;
    logic [31:0] state;
    localparam state_0 = 0;
    localparam state_1 = 1;
    localparam state_start = 2;
    localparam state_done = 3;
    always_ff @(posedge clock) begin
        if(ready || ~valid) begin
            case (state)
                state_start : begin
                    mem_0 <= n;
                    if(start) begin
                        state <= state_0;
                    end
                end
                state_0 : begin
                    n <= mem_0;
                    mem_0 <= 0;
                    mem_1 <= 1;
                    mem_2 <= 0;
                    if((0 < mem_0)) begin
                        if((0 % 2)) begin
                            valid <= 1;
                            out_0 <= 0;
                            mem_0 <= 1;
                            mem_1 <= (0 + 1);
                            mem_2 <= (0 + 1);
                            mem_3 <= mem_0;
                            state <= state_1;
                        end else if(!(0 % 2)) begin
                            mem_0 <= 1;
                            mem_1 <= (0 + 1);
                            mem_2 <= (0 + 1);
                            mem_3 <= mem_0;
                            state <= state_1;
                        end
                    end else if(!(0 < mem_0)) begin
                        valid <= 1;
                        done <= 1;
                        out_0 <= 0;
                    end
                end
                state_1 : begin
                    n <= mem_3;
                    i3 <= mem_2;
                    b3 <= mem_1;
                    a2 <= mem_0;
                    if((mem_0 < mem_3)) begin
                        if((mem_0 % 2)) begin
                            valid <= 1;
                            out_0 <= mem_0;
                            mem_0 <= mem_1;
                            mem_1 <= (mem_0 + mem_1);
                            mem_2 <= (mem_2 + 1);
                            mem_3 <= mem_3;
                            state <= state_1;
                        end else if(!(mem_0 % 2)) begin
                            mem_0 <= mem_1;
                            mem_1 <= (mem_0 + mem_1);
                            mem_2 <= (mem_2 + 1);
                            mem_3 <= mem_3;
                            state <= state_1;
                        end
                    end else if(!(mem_0 < mem_3)) begin
                        valid <= 1;
                        done <= 1;
                        out_0 <= 0;
                    end
                end
            endcase
        end
    end
endmodule