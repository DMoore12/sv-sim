module no_width_case_test
(
   input wire sel,
   output reg data_out
);

   always begin
      case (sel)
         0: data_out = 1;
         1: data_out = 2;
         default: data_out = 255;
      endcase
   end

endmodule
