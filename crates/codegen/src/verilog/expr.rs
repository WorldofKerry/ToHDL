use tohdl_ir::expr::Expr;

pub trait ToVerilog {
    fn to_verilog(&self) -> String;
}

impl ToVerilog for Expr {
    fn to_verilog(&self) -> String {
        match self {
            Expr::Var(_) => self.to_string(),
            Expr::Int(_) => format!("$signed({})", self),
            Expr::BinOp(_, _, _) => format!("$signed({})", self),
        }
    }
}
