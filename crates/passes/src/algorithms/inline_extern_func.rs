use tohdl_ir::graph::CallNode;
use tohdl_ir::graph::Node;
use tohdl_ir::graph::NodeIndex;
use tohdl_ir::graph::ReturnNode;
use tohdl_ir::graph::CFG;

/// Inlines an external function
pub fn inline_extern_func<'a>(extern_node: NodeIndex, caller: &mut CFG, callee: &CFG) {
    let old_to_new_idx = CFG::merge_graph(caller, callee);

    let callee_exits = CFG::find_exits(callee).collect::<Vec<_>>();
    let callee_exits = callee_exits
        .into_iter()
        .map(|x| old_to_new_idx.get(&x).unwrap())
        .collect::<Vec<_>>();
    let callee_entry = old_to_new_idx.get(&callee.entry).unwrap();

    let extern_preds = caller.preds(extern_node).collect::<Vec<_>>();
    let extern_succs = caller.succs(extern_node).collect::<Vec<_>>();
    assert_eq!(
        extern_succs.len(),
        1,
        "External node should only have one child being the func node"
    );

    // Connect call to entry
    for pred in extern_preds {
        let edge = caller.get_edge(pred, extern_node).unwrap();
        caller.add_edge(pred, callee_entry.clone(), edge.clone());
    }

    // Connect exits to func node
    for succ in extern_succs {
        for exit in &callee_exits {
            let edge = caller.get_edge(extern_node, succ).unwrap();
            caller.add_edge((*exit).clone(), succ.clone(), edge.clone());
        }
    }

    caller.rmv_node(extern_node);

    // Replace exit return nodes with call nodes
    for exit in callee_exits {
        if let Some(node) = ReturnNode::concrete(caller.get_node(*exit)) {
            let call_node = convert_return_to_call_node(node);
            caller.replace_node(*exit, call_node)
        } else {
            panic!(
                "Expected return node for function exit {}",
                caller.get_node(*exit)
            );
        }
    }
}

/// Converts return node to call node
fn convert_return_to_call_node(node: &ReturnNode) -> CallNode {
    let args = node
        .values
        .clone()
        .into_iter()
        .map(|x| match x {
            tohdl_ir::expr::Expr::Var(v) => v,
            _ => panic!("Expected VarExpr {}", x),
        })
        .collect();
    CallNode { args }
}
