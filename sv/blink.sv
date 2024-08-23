`timescale 1ns/1ps

module blink
(
    input wire clk,
    input wire n_rst,
    output reg led,
);
    wire next_state;

    // Assign next state to inverse of current state
    always_comb begin
        if (state == 1'b0) begin
            next_state = 1'b1;
        end else begin
            next_state = 1'b0;
        end
    end

    // Update led state
    always @ (posedge clk, negedge n_rst) begin
        if (n_rst == 1'b0) begin
            led <= 1'b0;
        end else begin
            led <= next_state;
        end
    end

endmodule