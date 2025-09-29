module simple_param_test
(
   input wire a,
   output wire b
);

   parameter WIDTH = 8;
   
   assign b = a;

endmodule
