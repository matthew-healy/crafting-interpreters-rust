use astgen::generate_ast;
use crate::{
    expr::Expr,
    token::Token,
};

generate_ast!(
    Stmt,
    [
        Block      => { statements: Vec<Stmt> };
        Class      => { name: Token, superclass: Option<Expr>, methods: Vec<Function> };
        Expression => { expression: Expr };
        Function   => { name: Token, params: Vec<Token>, body: Vec<Stmt> };
        If         => { condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> };
        Print      => { expression: Expr };
        Return     => { keyword: Token, value: Option<Expr> };
        Var        => { name: Token, initializer: Option<Expr> };
        While      => { condition: Expr, body: Box<Stmt> };
    ]
);