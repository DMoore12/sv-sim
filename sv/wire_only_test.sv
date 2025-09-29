module wire_only_test
(
   input wire sel,
   output wire data_out
);

   assign data_out = sel;

endmodule
