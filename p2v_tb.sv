/*
# Python Function
@verilogify
def even_fib(n):
    i = 0
    a = 0
    b = 1
    while a < n:
        if a % 2:
            yield a
        temp = a + b
        a = b
        b = temp
        i = i + 1
    # return 0

# Test Cases
print(list(even_fib(*(10,))))
*/
module even_fib_tb (
);
    reg __clock;
    reg __start;
    reg __reset;
    reg __ready;
    reg signed [31:0] n;
    wire __done;
    wire __valid;
    wire signed [31:0] __output_0;
    even_fib DUT (
        .__clock(__clock),
        .__start(__start),
        .__reset(__reset),
        .__ready(__ready),
        .n(n),
        .__done(__done),
        .__valid(__valid),
        .__output_0(__output_0)
        );
    always #5 __clock = !__clock;
    initial begin
        __clock = 0;
        __start = 0;
        __ready = 1;
        __reset = 1;
        @(negedge __clock);
        __reset = 0;
        // ============ Test Case 0 with arguments (10,) ============
        n = $signed(40);
        __start = 1;
        @(negedge __clock);
        n = 'x; // only need inputs when start is set
        __start = 0;
        while ((!(__done) || !(__ready))) begin
            @(posedge __clock);
            if (__ready) begin
                $display("%0d, %0d, %0d", __ready, __valid, __output_0);
            end
            @(negedge __clock);
        end
        $display("tb: start %0d, done %0d, ready %0d", __start, __done, __ready);
        $finish;
    end
endmodule

