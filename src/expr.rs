use std::fmt::Display;
use crate::token::Token;

macro_rules! generate_ast {
    ($($typename:ident => $($propname:ident: $proptype:ty),+);+) => {
        #[derive(Debug, PartialEq)]
        pub enum Expr {
            $($typename($typename)),+
        }

        $(
            #[derive(Debug, PartialEq)]
            pub struct $typename {
                $(pub(crate) $propname: $proptype),+
            }
        )+
    }
}

macro_rules! generate_visitor {
    ($($typename:ident => $visitname:ident);+) => {
        pub(crate) trait Visitor<T> {
            $(fn $visitname(&mut self, e: &$typename) -> T;)+
        }

        impl Expr {
            pub(crate) fn accept<T, V: Visitor<T>>(&self, v: &mut V) -> T {
                match self {
                    $(Expr::$typename(a) => v.$visitname(a),)+
                }
            }
        }
    };
}

generate_ast!(
    Binary => left: Box<Expr>, op: Token, right: Box<Expr>;
    Grouping => expression: Box<Expr>;
    Literal => value: LoxLiteral;
    Unary => op: Token, right: Box<Expr>
);

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum LoxLiteral {
    Bool(bool),
    Nil,
    Number(f64),
    String(String),
}

impl From<bool> for LoxLiteral {
    fn from(b: bool) -> Self {
        LoxLiteral::Bool(b)
    }
}

impl From<f64> for LoxLiteral {
    fn from(n: f64) -> Self {
        LoxLiteral::Number(n)
    }
}

impl Display for LoxLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LoxLiteral::*;
        match self {
            Bool(b) => write!(f, "{}", b),
            Nil => write!(f, "nil"),
            Number(n) => write!(f, "{}", n),
            String(s) => write!(f, "{}", s),
        }
    }
}

generate_visitor!(
    Binary => visit_binary_expr;
    Grouping => visit_grouping_expr;
    Literal => visit_literal_expr;
    Unary => visit_unary_expr
);

