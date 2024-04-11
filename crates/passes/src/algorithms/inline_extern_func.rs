use tohdl_ir::expr::Expr;
use tohdl_ir::graph::CallNode;
use tohdl_ir::graph::BranchEdge;
use tohdl_ir::graph::Node;
use tohdl_ir::graph::NodeIndex;
use tohdl_ir::graph::NoneEdge;
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

    // Handle return nodes with a non-variable expression in them
    // Inserts an assignnode into a temp var before it
    // E.g. return(5) becomes temp = 5 -> return(temp)
    for exit in &callee_exits {
        let mut added_nodes = vec![];
        if let Some(node) = ReturnNode::concrete_mut(caller.get_node_mut(**exit)) {
            if node.values.iter().any(|x| match x {
                tohdl_ir::expr::Expr::Var(_) => false,
                _ => true,
            }) {
                let mut added_vars = vec![];
                // Use node id as temp var name
                for (i, expr) in node.values.iter().enumerate() {
                    let temp_var = format!("temp_{}_{}", exit.0, i);
                    let temp_var = tohdl_ir::expr::VarExpr::new(&temp_var);
                    let assign_node = tohdl_ir::graph::AssignNode {
                        lvalue: temp_var.clone(),
                        rvalue: expr.clone(),
                    };
                    added_nodes.push(assign_node);
                    added_vars.push(Expr::Var(temp_var));
                }
                node.values = added_vars;
            }
        } else {
            panic!(
                "Expected return node for function exit {}",
                caller.get_node(**exit)
            );
        }
        for node in added_nodes {
            caller.insert_node_before(node, **exit, NoneEdge.into());
        }
    }

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
