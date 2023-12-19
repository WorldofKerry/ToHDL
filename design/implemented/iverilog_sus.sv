module tb();
  reg signed [31:0] x;
  reg signed [31:0] y;
  reg signed [31:0] a;
  reg signed [31:0] b;

  initial begin
    x = 10;
    y = -1;

    $display("%d %d %d %d %d %d", x, y, a, b, (x / y * y == x), x / y * y);
    a = ((x / y * y == x) ? (x / y) : (0 < 1));

    $display("%d %d %d %d %d %d", x, y, a, b, (x / y * y == x), x / y * y);
    b = ((x / y * y == x) ? (x / y) : (123));

    $display("%d %d %d %d %d %d", x, y, a, b, (x / y * y == x), x / y * y);

  end
endmodule
