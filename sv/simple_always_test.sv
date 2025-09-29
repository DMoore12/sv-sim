module simple_always_test
(
   input wire a,
   output reg b
);

   always begin
      b = a;
   end

endmodule
