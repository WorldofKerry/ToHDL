use tohdl_ir::graph::NodeIndex;
use tohdl_ir::graph::CFG;

/// Inlines an external function
pub fn inline_extern_func<'a>(
    call_node: NodeIndex,
    func_node: NodeIndex,
    caller: &mut CFG,
    callee: &CFG,
) {

}
