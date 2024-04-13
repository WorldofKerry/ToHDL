use std::collections::BTreeMap;

use pytohdl::{find_externals, PyContext};
use tohdl_codegen::verilog::graph_to_verilog;
use tohdl_ir::graph::CFG;
use tohdl_passes::{
    algorithms::inline_extern_func, manager::PassManager, transform::{BraunEtAl, InsertCallNodes, InsertFuncNodes}, BasicTransform
};
use tohdl_tests::{aug_assign_str, binary_to_7_seg_str, div_10_str, fib_to_7_seg_str, func_call_str, mod_10_str, return_literal_str, seven_seg_str};

fn aug_assign_graph() -> CFG {
    let mut graph = tohdl_tests::aug_assign_graph();

    InsertFuncNodes::default().apply(&mut graph);
    InsertCallNodes::default().apply(&mut graph);

    let mut pass = BraunEtAl::default();

    pass.apply(&mut graph);
    graph
}

#[test]
fn aug_assign() {
    let graph = aug_assign_graph();
    graph.write_dot("output.dot");
}

#[test]
fn func_call() {
    let mut graph = tohdl_tests::func_call_graph();
    let pycontext = PyContext {
        main: "func_call".into(),
        functions: BTreeMap::from([
            ("func_call".into(), func_call_str().into()),
            ("aug_assign".into(), aug_assign_str().into()),
            ("return_literal".into(), return_literal_str().into()),
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
    }

    let mut manager = PassManager::log();
    manager.add_pass(InsertFuncNodes::transform);
    manager.add_pass(InsertCallNodes::transform);
    manager.add_pass(BraunEtAl::transform);
    manager.apply(&mut graph);
    graph.write_dot("output.dot");
}

#[ignore]
#[test]
fn fib_to_7_seg() {
    let mut graph = tohdl_frontend::AstVisitor::from_text(tohdl_tests::binary_to_7_seg_str()).get_graph();
    let pycontext = PyContext {
        main: "binary_to_7_seg".into(),
        functions: BTreeMap::from([
            ("fib_to_7_seg".into(), fib_to_7_seg_str().into()),
            ("binary_to_7_seg".into(), binary_to_7_seg_str().into()),
            ("mod_10".into(), mod_10_str().into()),
            ("div_10".into(), div_10_str().into()),
            ("seven_seg".into(), seven_seg_str().into()),
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
