module integer_case_test
(
   input wire [1:0] sel,
   output reg [7:0] data_out
);

   always begin
      case (sel)
         0: data_out = 1;
         1: data_out = 2;
         default: data_out = 255;
      endcase
   end

endmodule
