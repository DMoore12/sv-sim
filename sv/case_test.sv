module case_test
(
   input wire [2:0] sel,
   input wire [7:0] data_in,
   output reg [7:0] data_out
);

   parameter NUM_CHANNELS = 8;
   
   always @(*) begin
      case (sel)
         3'b000: data_out = data_in + 8'h01;
         3'b001: data_out = data_in + 8'h02;
         3'b010: data_out = data_in + 8'h03;
         default: data_out = 8'hFF;
      endcase
   end

endmodule
