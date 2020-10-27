use astgen::generate_ast;
use crate::expr::Expr;

generate_ast!(
    Stmt,
    [
        Expression => { expression: Expr };
        Print => { expression: Expr };
    ]
);