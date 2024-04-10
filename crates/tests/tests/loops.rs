use std::collections::BTreeMap;

use pytohdl::{find_externals, PyContext};
use tohdl_codegen::verilog::graph_to_verilog;
use tohdl_ir::graph::CFG;
use tohdl_passes::{
    algorithms::inline_extern_func,
    transform::{BraunEtAl, InsertCallNodes, InsertFuncNodes},
    Transform,
};
use tohdl_tests::{aug_assign_str, binary_to_7_seg_str, div_10_str, fib_to_7_seg_str, func_call_str, mod_10_str, return_literal_str, seven_seg_str, while_loop_graph};

#[test]
fn aug_assign() {
    let graph = while_loop_graph();
    graph.write_dot("output.dot");
}
