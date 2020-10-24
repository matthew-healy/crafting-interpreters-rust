use crate::token::Token;

macro_rules! generate_ast {
    ($($typename:ident => $($propname:ident: $proptype:ty),+);+) => {
        pub(crate) enum Expr {
            $($typename($typename)),+
        }

        $(
            pub(crate) struct $typename {
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

generate_visitor!(
    Binary => visit_binary_expr;
    Grouping => visit_grouping_expr;
    Literal => visit_literal_expr;
    Unary => visit_unary_expr
);

