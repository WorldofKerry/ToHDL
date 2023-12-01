use tohdl_ir::expr::VarExpr;
use vast::v17::ast::{self as v, Sequential};

pub fn create_memories(max_memory: usize) -> Vec<v::Stmt> {
    (0..max_memory)
        .map(|i| v::Stmt::new_decl(v::Decl::new_logic(&format!("mem_{}", i), 32)))
        .collect()
}

#[cfg(test)]
mod test {
    use tohdl_passes::{
        manager::PassManager,
        optimize::RemoveUnreadVars,
        transform::{BraunEtAl, InsertCallNodes, InsertFuncNodes, Nonblocking},
        Transform,
    };
    use vast::v05::ast::CaseBranch;

    use crate::{tests::make_odd_fib, verilog::SingleStateLogic};

    use super::*;

    #[test]
    fn main() {
        let result = create_memories(10);
        println!("{:?}", result);
    }
}
