pub mod expr;
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
    manager::PassManager, optimize::RemoveUnreadVars, transform::{
        BraunEtAl, ExplicitReturn, FixBranch, InsertCallNodes, InsertFuncNodes, Nonblocking,
    }, BasicTransform, ContextfulTransfrom, TransformResultType
};

pub fn graph_to_verilog(mut graph: CFG) -> String {
    let mut manager = PassManager::log();
    manager.add_pass(InsertFuncNodes::transform);
    manager.add_pass(InsertCallNodes::transform);
    manager.add_pass(BraunEtAl::transform);
    manager.apply(&mut graph);

    let mut lower = tohdl_passes::transform::LowerToFsm::default();
    let result = lower.apply_timed(&mut graph);
    println!("{result}");

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
        // subgraph.write_dot(format!("debug_{}.dot", i).as_str());
        let max_memory = {
            let mut pass = crate::verilog::UseMemory::default();
            let result = pass.apply_timed(&mut subgraph);
            println!("{result}");
            pass.max_memory()
        };

        let mut manager = PassManager::log();
        manager.add_pass(Nonblocking::transform);
        manager.add_pass(RemoveLoadsEtc::transform);
        manager.add_pass(RemoveUnreadVars::transform);
        manager.add_pass(FixBranch::transform);
        manager.add_pass(ExplicitReturn::transform);
        manager.apply(&mut subgraph);

        context.memories.count = std::cmp::max(context.memories.count, max_memory);

        let mut codegen = SingleStateLogic::new(lower.get_external_funcs(i));
        let result = codegen.apply_timed_contextful(&mut subgraph, &mut context);
        println!("{result}");
        states.push(codegen);

    }

    let module = new_create_module(states, &context);
    format!("{}", module)
}
