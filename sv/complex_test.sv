`timescale 1ns/1ps

module complex_test
(
   input wire [3:0] a,
   input wire [3:0] b,
   output wire [3:0] sum,
   output wire carry
);

   assign sum = a + b;
   assign carry = (a + b) > 4'hF;

endmodule
