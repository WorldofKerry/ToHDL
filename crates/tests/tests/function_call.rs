use tohdl_ir::graph::CFG;
use tohdl_passes::{
    algorithms::inline_extern_func,
    transform::{BraunEtAl, InsertCallNodes, InsertFuncNodes},
    Transform,
};

fn aug_assign_graph() -> CFG {
    let mut graph = tohdl_tests::make_aug_assign();

    InsertFuncNodes::default().apply(&mut graph);
    InsertCallNodes::default().apply(&mut graph);

    let mut pass = BraunEtAl::default();

    pass.apply(&mut graph);
    graph
}

#[test]
fn aug_assign() {
    let graph = aug_assign_graph();
    // println!(
    //     "final {}",
    //     pass.read_variable(&mut graph, &VarExpr::new("a"), &5.into())
    // );
    // println!(
    //     "final {}",
    //     pass.read_variable(&mut graph, &VarExpr::new("d"), &5.into())
    // );

    // println!("read_vars {:?}", pass.read_vars);
    // println!("wrote_vars {:?}", pass.wrote_vars);
    // println!("current_def {:#?}", pass.current_def);

    graph.write_dot("output.dot");
}

#[test]
fn func_call() {
    let mut graph = tohdl_tests::make_func_call();

    let callee_graph = aug_assign_graph();
    inline_extern_func(3.into(), &mut graph, &callee_graph);

    InsertFuncNodes::default().apply(&mut graph);
    InsertCallNodes::default().apply(&mut graph);
    let mut pass = BraunEtAl::default();
    pass.apply(&mut graph);

    // println!(
    //     "final {}",
    //     pass.read_variable(&mut graph, &VarExpr::new("a"), &5.into())
    // );
    // println!(
    //     "final {}",
    //     pass.read_variable(&mut graph, &VarExpr::new("d"), &5.into())
    // );

    // println!("read_vars {:?}", pass.read_vars);
    // println!("wrote_vars {:?}", pass.wrote_vars);
    // println!("current_def {:#?}", pass.current_def);

    graph.write_dot("output.dot");
}
