use crate::token::Token;
use crate::value::Value;
use astgen::generate_ast;

generate_ast!(
    Expr,
    [
        Binary => { left: Box<Expr>, op: Token, right: Box<Expr> };
        Grouping => { expression: Box<Expr> };
        Literal => { value: Value };
        Unary => { op: Token, right: Box<Expr> };
        Variable => { name: Token };
    ]
);