module debug_test
(
   input wire a,
   output reg b
);

   always begin
      if (a) begin
         b = 1;
      end else begin
         b = 0;
      end
   end

endmodule
