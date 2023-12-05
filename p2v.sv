module even_fib (
    input logic [31:0] n,
    input logic __ready,
    input logic __start,
    input logic __clock,
    input logic __reset,
    output logic __valid,
    output logic __done,
    output logic [31:0] __output_0
);
    localparam state_0 = 0;
    localparam state_1 = 1;
    localparam state_start = 2;
    localparam state_done = 3;
    logic [31:0] mem_0;
    logic [31:0] mem_1;
    logic [31:0] mem_2;
    logic [31:0] mem_3;
    logic [31:0] state;
    always_ff @(posedge __clock) begin
        __done <= 0;
        if(__reset) begin
            state <= state_start;
        end
        if(__ready || ~__valid) begin
            __valid <= 0;
            case (state)
                state_start : begin
                    mem_0 <= n;
                    if(__start) begin
                        state <= state_0;
                    end
                end
                state_done : begin
                    __done <= 1;
                    state <= state_start;
                end
                state_0 : begin
                    if((0 < mem_0)) begin
                        if((0 % 2)) begin
                            __valid <= 1;
                            __output_0 <= 0;
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
                        __valid <= 1;
                        __output_0 <= 0;
                        state <= state_done;
                    end
                end
                state_1 : begin
                    if((mem_0 < mem_3)) begin
                        if((mem_0 % 2)) begin
                            __valid <= 1;
                            __output_0 <= mem_0;
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
                        __valid <= 1;
                        __output_0 <= 0;
                        state <= state_done;
                    end
                end
            endcase
        end
    end
endmodule