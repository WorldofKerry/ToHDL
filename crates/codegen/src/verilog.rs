mod memory;
pub use memory::*;
mod state;
pub use state::*;
mod module;
pub use module::*;
mod helpers;
pub use helpers::*;
mod clean_assignments;
pub use clean_assignments::*;
use tohdl_ir::graph::CFG;
use tohdl_passes::{
    manager::PassManager,
    optimize::RemoveUnreadVars,
    transform::{
        BraunEtAl, ExplicitReturn, FixBranch, InsertCallNodes, InsertFuncNodes, Nonblocking,
    },
    Transform,
};

pub fn graph_to_verilog(mut graph: CFG) -> String {
    let mut manager = PassManager::default();
    
    manager.add_pass(InsertFuncNodes::transform);
    manager.add_pass(InsertCallNodes::transform);
    manager.add_pass(BraunEtAl::transform);
    
    manager.apply(&mut graph);
    
    graph.write_dot("./original.dot");

    // return format!("");
    let mut lower = tohdl_passes::transform::LowerToFsm::default();
    lower.apply(&mut graph);

    let mut states = vec![];

    let signals = Signals::new();
    let mut context = Context::new(
        graph.name.as_str(),
        graph.get_inputs().cloned().collect(),
        signals,
    );

    // Write all new subgraphs to files
    for (i, subgraph) in lower.get_subgraphs().iter().enumerate() {
        let mut subgraph = subgraph.clone();
        let max_memory = {
            let mut pass = crate::verilog::UseMemory::default();
            pass.apply(&mut subgraph);
            pass.max_memory()
        };
        Nonblocking::transform(&mut subgraph);
        RemoveLoadsEtc::transform(&mut subgraph);
        RemoveUnreadVars::transform(&mut subgraph);
        FixBranch::transform(&mut subgraph);
        ExplicitReturn::transform(&mut subgraph);
        subgraph.write_dot(format!("debug_{}.dot", i).as_str());
        context.memories.count = std::cmp::max(context.memories.count, max_memory);

        let mut codegen = SingleStateLogic::new(subgraph, i, lower.get_external_funcs(i));
        codegen.apply(&mut context);
        // println!("codegen body {:?}", codegen.body);
        states.push(codegen);
    }

    let body = create_module_body(states, &context);
    let module = create_module(body, &context);
    format!("{}", module)
}
