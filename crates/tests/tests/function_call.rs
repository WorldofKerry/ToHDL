use tohdl_passes::{transform::{BraunEtAl, InsertCallNodes, InsertFuncNodes}, Transform};

#[test]
fn func_call() {
    let mut graph = tohdl_tests::make_func_call();

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
