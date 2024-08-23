// $Id: $
// File name:   flex_counter.sv
// Created:     2/18/2021
// Author:      Dawson Moore
// Lab Section: 337-04
// Version:     1.0  Initial Design Entry
// Description: 
//

`timescale 1ns / 10ps

module flex_counter
#(
    parameter NUM_CNT_BITS = 4
)
(
    input wire clk,
    input wire n_rst,
    input wire clear,
    input wire count_enable,
    input wire [NUM_CNT_BITS - 1:0] rollover_val,
    output reg [NUM_CNT_BITS - 1:0] count_out,
    output reg rollover_flag
);

    reg [NUM_CNT_BITS - 1:0] next_count;
    reg next_rollover;

    always_comb begin
        next_count = count_out;
        next_rollover = rollover_flag;
        if (clear) begin
            next_count = 0;
            next_rollover = 0;
        end else if (count_enable) begin
            if (count_out == rollover_val) begin
                next_count = 1;
                next_rollover = 0;
            end else begin
                next_count = count_out + 1;
                if (next_count == rollover_val) begin
                    next_rollover = 1;
                end else begin
                    next_rollover = 0;
                end
            end
        end
    end

    always_ff @ (posedge clk, negedge n_rst) begin
        if (n_rst == 1'b0) begin
            count_out <= 0;
            rollover_flag <= 0;
        end else begin
            count_out <= next_count;
            rollover_flag <= next_rollover;
        end
    end

endmodule