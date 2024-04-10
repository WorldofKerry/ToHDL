use std::collections::BTreeMap;

use pytohdl::{find_externals, PyContext};
use tohdl_ir::graph::CFG;
use tohdl_passes::{
    algorithms::inline_extern_func,
    transform::{BraunEtAl, InsertCallNodes, InsertFuncNodes},
    Transform,
};
use tohdl_tests::{aug_assign_str, func_call_str, return_literal_str};

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

    InsertFuncNodes::default().apply(&mut graph);
    InsertCallNodes::default().apply(&mut graph);
    let mut pass = BraunEtAl::default();
    pass.apply(&mut graph);
    graph.write_dot("output.dot");
}
