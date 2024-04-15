use std::collections::BTreeMap;

use pytohdl::{find_externals, translate, PyContext};
use tohdl_codegen::verilog::graph_to_verilog;
use tohdl_ir::graph::CFG;
use tohdl_passes::{
    algorithms::inline_extern_func, manager::PassManager, transform::{BraunEtAl, InsertCallNodes, InsertFuncNodes}, BasicTransform
};
use tohdl_tests::{aug_assign_str, binary_to_7_seg_str, div_10_str, fib_to_7_seg_str, func_call_str, mod_10_str, return_literal_str, seven_seg_str, while_loop_graph};

#[test]
fn loops() {
    let pycontext = PyContext {
        main: "while_loop".into(),
        functions: BTreeMap::from([
            ("while_loop".into(), tohdl_tests::while_loop_str().into()),
            // ("fib_to_7_seg".into(), fib_to_7_seg_str().into()),
            // ("binary_to_7_seg".into(), binary_to_7_seg_str().into()),
            // ("mod_10".into(), mod_10_str().into()),
            // ("div_10".into(), div_10_str().into()),
            // ("seven_seg".into(), seven_seg_str().into()),
            // ("return_literal".into(), return_literal_str().into()),
        ])
        .into(),
    };
    let code = translate(&pycontext);
}
