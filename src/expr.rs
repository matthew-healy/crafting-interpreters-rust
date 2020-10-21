use crate::token::Token;

macro_rules! generate_ast {
    ($($typename:ident => $($propname:ident: $proptype:ty),+);+) => {
        enum Expr {
            $($typename($typename)),+
        }

        $(
            struct $typename {
                $($propname: $proptype),+
            }
        )+
    }
}

macro_rules! generate_visitor {
    ($($typename:ident => $visitname:ident);+) => {
        trait Visitor<T> {
            $(fn $visitname(&mut self, e: &$typename) -> T;)+
        }

        impl Expr {
            fn accept<T, V: Visitor<T>>(&self, v: &mut V) -> T {
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
    Unary => op: Token, right: Box<Expr>;
    NumberLiteral => value: f64;
    StringLiteral => value: String
);

generate_visitor!(
    Binary => visit_binary_expr;
    Grouping => visit_grouping_expr;
    NumberLiteral => visit_number_literal_expr;
    StringLiteral => visit_string_literal_expr;
    Unary => visit_unary_expr
);

