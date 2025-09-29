module simple_case_test
(
   input wire [1:0] sel,
   input wire [7:0] data_in,
   output reg [7:0] data_out
);

   always begin
      case (sel)
         2'b00: data_out = data_in + 8'h01;
         2'b01: data_out = data_in + 8'h02;
         default: data_out = 8'hFF;
      endcase
   end

endmodule
