use crate::token::Token;
use crate::value;
use astgen::generate_ast;

generate_ast!(
    Expr,
    [
        Assign   => { name: Token, value: Box<Expr> };
        Binary   => { left: Box<Expr>, op: Token, right: Box<Expr> };
        Call     => { callee: Box<Expr>, paren: Token, arguments: Vec<Expr> };
        Grouping => { expression: Box<Expr> };
        Literal  => { value: value::Literal };
        Logical  => { left: Box<Expr>, op: Token, right: Box<Expr> };
        Unary    => { op: Token, right: Box<Expr> };
        Variable => { name: Token };
    ]
);