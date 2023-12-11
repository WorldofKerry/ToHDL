use tohdl_ir::expr::Expr;

pub trait ToVerilog {
    fn to_verilog(&self) -> String;
}

impl ToVerilog for Expr {
    fn to_verilog(&self) -> String {
        match self {
            Expr::Var(_) => self.to_string(),
            Expr::Int(_) => self.to_string(),
            Expr::BinOp(left, op, right) => format!("$signed({})", self.to_string()),
        }
    }
}
