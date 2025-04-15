use tohdl_ir::expr::Expr;

pub trait ToVerilog {
    fn to_verilog(&self) -> String;
}

impl ToVerilog for Expr {
    fn to_verilog(&self) -> String {
        match self {
            Expr::Var(_) => format!("$signed({})", self),
            Expr::Int(_) => format!("$signed({})", self),
            Expr::BinOp(left, op, right) => format!("$signed({} {} {})", left.to_verilog(), op, right.to_verilog()),
        }
    }
}
