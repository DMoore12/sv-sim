module advanced_test
(
   input wire clk,
   input wire rst_n,
   input wire [2:0] sel,
   input wire [7:0] data_in,
   output reg [7:0] data_out,
   output reg valid
);

   parameter NUM_CHANNELS = 8;
   localparam ADDR_WIDTH = 3;
   
   reg [7:0] registers [0:NUM_CHANNELS-1];
   integer i;

   always @(posedge clk) begin
      if (!rst_n) begin
         data_out <= 8'h00;
         valid <= 1'b0;
         for (i = 0; i < NUM_CHANNELS; i = i + 1) begin
            registers[i] <= 8'h00;
         end
      end else begin
         case (sel)
            3'b000: begin
               registers[0] <= data_in;
               data_out <= registers[0];
            end
            3'b001: begin
               registers[1] <= data_in;
               data_out <= registers[1];
            end
            3'b010: begin
               registers[2] <= data_in;
               data_out <= registers[2];
            end
            default: begin
               data_out <= 8'hFF;
            end
         endcase
         valid <= 1'b1;
      end
   end

endmodule
