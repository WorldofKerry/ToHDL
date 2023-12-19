module tb();
  reg signed [31:0] x;
  reg signed [31:0] y;
  reg signed [31:0] z;
  reg signed [31:0] a;
  reg signed [31:0] b;

  initial begin
    x = 1;
    y = -1;

//     z = (x >= $signed(0)) ? (x / y) : (x / (y - $signed(-1)));
//     z = x / (y - 50);
//     z = x / y >= $signed(0) ? x / y :
//     	x / (y - $signed(1));
//     (y - $signed(1) == 0 ? x : x / (y - $signed(1)));

    //     z = x / y >= $signed(0) ? x % y : -(x % y);

//     z = x / y;

//     z = x / y;
//     z = z * y == x ? z : z - ((x < 0) ^ (y < 0));

    a = (x / y * y == x) ? (x / y) : (x / y - ((x < 0) ^ (y < 0)));

    b = (x / y * y == x) ? (x / y) : 123;


    $display("%d %d %d %d", x, y, a, b);

  end
endmodule
