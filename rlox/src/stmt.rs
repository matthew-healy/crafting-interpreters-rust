use astgen::generate_ast;
use crate::{
    expr::Expr,
    token::Token,
};

generate_ast!(
    Stmt,
    [
        Block      => { statements: Vec<Stmt> };
        Expression => { expression: Expr };
        If         => { condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> };
        Print      => { expression: Expr };
        Var        => { name: Token, initializer: Option<Expr> };
    ]
);