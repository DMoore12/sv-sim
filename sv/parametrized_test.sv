`timescale 1ns/1ps

module parametrized_test
#(
    parameter WIDTH = 8,
    parameter DEPTH = 16
)
(
   input wire clk,
   input wire rst_n,
   input wire [WIDTH-1:0] data_in,
   output reg [WIDTH-1:0] data_out,
   output reg valid
);

   localparam HALF_WIDTH = WIDTH / 2;
   
   reg [WIDTH-1:0] memory [0:DEPTH-1];
   reg [3:0] counter;

   always @(posedge clk) begin
      if (!rst_n) begin
         data_out <= {WIDTH{1'b0}};
         valid <= 1'b0;
         counter <= 4'b0000;
      end else begin
         memory[counter] <= data_in;
         data_out <= memory[counter];
         valid <= 1'b1;
         counter <= counter + 1;
      end
   end

endmodule
