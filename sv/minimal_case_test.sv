module minimal_case_test
(
   input wire a,
   output reg b
);

   always @(*) begin
      b = a;
   end

endmodule
