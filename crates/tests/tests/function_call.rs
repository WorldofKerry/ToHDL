use tohdl_ir::graph::CFG;
use tohdl_passes::{
    algorithms::inline_extern_func,
    transform::{BraunEtAl, InsertCallNodes, InsertFuncNodes},
    Transform,
};

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

    let callee_graph = aug_assign_graph();
    inline_extern_func(3.into(), &mut graph, &callee_graph);

    InsertFuncNodes::default().apply(&mut graph);
    InsertCallNodes::default().apply(&mut graph);
    let mut pass = BraunEtAl::default();
    pass.apply(&mut graph);
    graph.write_dot("output.dot");
}
