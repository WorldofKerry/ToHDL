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
fn loops() {
    let mut graph = tohdl_frontend::AstVisitor::from_text(tohdl_tests::while_loop_str()).get_graph();
    let pycontext = PyContext {
        main: "while_loop".into(),
        functions: BTreeMap::from([
            // ("fib_to_7_seg".into(), fib_to_7_seg_str().into()),
            // ("binary_to_7_seg".into(), binary_to_7_seg_str().into()),
            // ("mod_10".into(), mod_10_str().into()),
            // ("div_10".into(), div_10_str().into()),
            // ("seven_seg".into(), seven_seg_str().into()),
            // ("return_literal".into(), return_literal_str().into()),
        ])
        .into(),
    };

    loop {
        let externals = find_externals(&graph, &pycontext);
        if externals.len() == 0 {
            break;
        }
        for (idx, callee_graph) in externals {
            inline_extern_func(idx, &mut graph, &callee_graph);
        }
        graph.write_dot("output.dot");
    }

    InsertFuncNodes::default().apply(&mut graph);
    InsertCallNodes::default().apply(&mut graph);
    let mut pass = BraunEtAl::default();
    pass.apply(&mut graph);
    graph.write_dot("output.dot");
    let code = graph_to_verilog(graph);
}
