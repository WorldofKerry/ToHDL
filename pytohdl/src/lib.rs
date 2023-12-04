use pyo3::prelude::*;
use tohdl_codegen::verilog::{
    create_module, create_module_body, Context, RemoveLoadsEtc, Signals, SingleStateLogic,
};
use tohdl_passes::{manager::PassManager, optimize::RemoveUnreadVars, transform::*, Transform};

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn pytohdl(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(translate, m)?)?;
    Ok(())
}

#[pyfunction]
fn translate(code: &str) -> String {
    let visitor = tohdl_frontend::AstVisitor::from_text(code);
    let mut graph = visitor.get_graph();

    let mut manager = PassManager::default();

    manager.add_pass(InsertFuncNodes::transform);
    manager.add_pass(InsertCallNodes::transform);
    manager.add_pass(BraunEtAl::transform);

    manager.apply(&mut graph);

    let mut lower = tohdl_passes::transform::LowerToFsm::default();
    lower.apply(&mut graph);

    let mut states = vec![];

    let signals = Signals::new();
    let mut context = Context::new("fib", graph.get_inputs().cloned().collect(), signals);

    // Write all new subgraphs to files
    for (i, subgraph) in lower.get_subgraphs().iter().enumerate() {
        let mut subgraph = subgraph.clone();
        let max_memory = {
            let mut pass = tohdl_codegen::verilog::UseMemory::default();
            pass.apply(&mut subgraph);
            pass.max_memory()
        };
        Nonblocking::transform(&mut subgraph);
        // RemoveAssignNodes::transform(&mut subgraph);
        RemoveLoadsEtc::transform(&mut subgraph);
        RemoveUnreadVars::transform(&mut subgraph);
        context.memories.count = std::cmp::max(context.memories.count, max_memory);

        subgraph.write_dot(format!("debug_{}.dot", i).as_str());
        let mut codegen = SingleStateLogic::new(subgraph, i, lower.get_external_funcs(i));
        codegen.apply(&mut context);
        states.push(codegen);
    }

    let body = create_module_body(states, &context);
    let module = create_module(body, &context);
    format!("{}", module)
}
