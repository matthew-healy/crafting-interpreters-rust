use proc_macro::TokenStream;
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident, Token, Type,
};
/// Parses the following syntax:
/// generate_ast!(
///     $AST_NAME,
///     [$(NODE_NAME => $($FIELD_NAME: $FIELD_TYPE),+)+])
/// )
//
/// For example:
///
/// generate_ast!(
///     Expr,
///     [
///         Number => value: isize;
///         Binary => left: Box<Expr>, op: char, right: Box<Expr>;
///     ]
/// )
struct Ast {
    name: Ident,
    nodes: Punctuated<AstNode, Token![;]>,
}

impl Parse for Ast {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let nodes_input;
        syn::bracketed!(nodes_input in input);
        let nodes: Punctuated<AstNode, Token![;]> = nodes_input.parse_terminated(AstNode::parse)?;
        Ok(Ast { name, nodes })
    }
}

struct AstNode {
    name: Ident,
    properties: Punctuated<Field, Token![,]>,
}

impl Parse for AstNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let properties = input.parse_terminated(Field::parse)?;
        Ok(AstNode { name, properties })
    }
}

struct Field {
    name: Ident,
    ty: Type,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        Ok(Field { name, ty })
    }
}

#[proc_macro]
pub fn generate_ast(input: TokenStream) -> TokenStream {
    let Ast {
        name,
        nodes: _,
    } = syn::parse_macro_input!(input);

    format!(
        "\
            enum {} {{\
                S(S),\
            }}\
            \
            struct S {{\
                s: String,\
            }}\
        ", 
        name.to_string(),
    ).parse().expect("generate_ast! failed to parse")
}