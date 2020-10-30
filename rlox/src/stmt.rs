use astgen::generate_ast;
use crate::{
    expr::Expr,
    token::Token,
};

generate_ast!(
    Stmt,
    [
        Expression => { expression: Expr };
        Print => { expression: Expr };
        Var => { name: Token, initializer: Expr };
    ]
);