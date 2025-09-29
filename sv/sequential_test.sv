`timescale 1ns/1ps

module sequential_test
(
   input wire clk,
   input wire rst_n,
   input wire [3:0] data_in,
   output reg [3:0] data_out,
   output reg valid
);

   always @(posedge clk) begin
      if (!rst_n) begin
         data_out <= 4'b0000;
         valid <= 1'b0;
      end else begin
         data_out <= data_in;
         valid <= 1'b1;
      end
   end

endmodule
