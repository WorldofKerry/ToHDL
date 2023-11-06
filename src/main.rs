fn main() {
    struct Operation {
        operator: Operator,
    }

    impl Iterator for Operation {
        type Item = Operator;

        fn next(&mut self) -> Option<Self::Item> {
            match self.operator {
                Operator::Add => {
                    self.operator = Operator::Sub;
                    Some(Operator::Add)
                }
                Operator::Sub => {
                    self.operator = Operator::Mul;
                    Some(Operator::Sub)
                }
                Operator::Mul => {
                    self.operator = Operator::Div;
                    Some(Operator::Mul)
                }
                Operator::Div => {
                    self.operator = Operator::Lt;
                    Some(Operator::Div)
                }
                Operator::Lt => {
                    self.operator = Operator::Gt;
                    Some(Operator::Lt)
                }
                Operator::Gt => None,
            }
        }
    }
}
