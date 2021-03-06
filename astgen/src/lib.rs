use proc_macro::TokenStream;
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident, Token, Type,
};
use quote::quote;
use heck::SnakeCase;

/// Parses the following syntax:
/// ```text
/// generate_ast!(
///     $AST_NAME,
///     [$(NODE_NAME => { $($FIELD_NAME: $FIELD_TYPE),+)+ }])
/// )
/// ```
///
/// For example:
/// ```text
/// generate_ast!(
///     Expr,
///     [
///         Number => { value: isize };
///         Binary => { left: Box<Expr>, op: char, right: Box<Expr> };
///     ]
/// )
/// ```
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
    fields: Punctuated<Field, Token![,]>,
}

impl Parse for AstNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let fields_input;
        syn::braced!(fields_input in input);
        let fields = fields_input.parse_terminated(Field::parse)?;
        Ok(AstNode { name, fields })
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

/// Generates an AST for the provided input. This includes a "top level"
/// enum, with a case for each node type, new_{node} functions for each
/// node, as well as a visitor trait with a visit function per node.
///
/// Example: the following invocation:
/// ```text
/// generate_ast!(
///     Expr,
///     [
///         Binary  => { left: Box<Expr>, op: Token, right: Box<Expr> };
///         Literal => { value: usize };
///     ]
/// );
/// ```
/// will generate code corresponding to:
/// ```text
/// #[derive(Clone, Debug, Eq, Hash, PartialEq)]
/// pub enum Expr {
///     Binary(Binary),
///     Literal(Literal),
/// }
///
/// #[derive(Clone, Debug, Eq, Hash, PartialEq)]
/// pub struct Binary {
///     pub(crate) left: Box<Expr>,
///     pub(crate) op: Token,
///     pub(crate) right: Box<Expr>,
/// }
///
/// #[derive(Clone, Debug, Eq, Hash, PartialEq)]
/// pub struct Literal {
///     pub(crate) value: usize,
/// }
///
/// impl Expr {
///     pub(crate) fn new_binary(left: Box<Expr>, op: Token, right: Box<Expr>) -> Self {
///         Self::Binary(Binary { left, op, right })
///     }
///
///     pub(crate) fn new_literal(value: usize) -> Self {
///         Self::Literal(Literal { value })
///     }
/// }
///
/// trait Visitor<T> {
///     fn visit_binary_expr(&mut self, e: &Binary) -> T;
///     fn visit_literal_expr(&mut self, e: &Literal) -> T;
/// }
///
/// impl Expr {
///     fn accept<T, V: Visitor<T>>(&self, v: &mut V) -> T {
///         match self {
///             Binary(b) => v.visit_binary_expr(b),
///             Literal(l) => v.visit_literal_expr(l),
///         }
///     }
/// }
/// ```
///
#[proc_macro]
pub fn generate_ast(input: TokenStream) -> TokenStream {
    let Ast {
        name,
        nodes,
    } = syn::parse_macro_input!(input);

    let lowercase_name = name.to_string().to_lowercase();
    let (node_names, visit_names): (Vec<_>, Vec<_>) = nodes.iter().map(|n| {
        let visit_name = quote::format_ident!("visit_{}_{}", n.name.to_string().to_snake_case(), lowercase_name);
        let name = &n.name;
        (name, visit_name)
    }).unzip();

    let ast_enum = quote! {
        #[derive(Clone, Debug, Eq, Hash, PartialEq)]
        pub enum #name {
            #(#node_names(#node_names)),*
        }
    };

    let node_structs = nodes.iter().map(|n| {
        let node_name = &n.name;
        let field_names = n.fields.iter().map(|f| &f.name);
        let field_types = n.fields.iter().map(|f| &f.ty);
        quote! {
            #[derive(Clone, Debug, Eq, Hash, PartialEq)]
            pub struct #node_name {
                #(pub(crate) #field_names: #field_types),*
            }
        }
    });

    let constructor_fns = nodes.iter().map(|n| {
        let enum_case = &n.name;
        let struct_name = &n.name;
        let snake_node_name = quote::format_ident!("new_{}", &n.name.to_string().to_snake_case());
        let arg_names = n.fields.iter().map(|f| &f.name);
        let field_names = arg_names.clone();
        let field_types = n.fields.iter().map(|f| &f.ty);
        quote! {
            pub(crate) fn #snake_node_name(#(#arg_names: #field_types),*) -> Self {
                Self::#enum_case(#struct_name {
                    #(#field_names),*
                })
            }
        }
    });

    let enum_impl = quote! {
        impl #name {
            #(#constructor_fns)*
        }
    };

    let visitor = quote! {
        pub(crate) trait Visitor<T> {
            #(fn #visit_names(&mut self, e: &#node_names) -> T;)*
        }

        impl #name {
            pub(crate) fn accept<T, V: Visitor<T>>(&self, v: &mut V) -> T {
                match self {
                    #(#name::#node_names(a) => v.#visit_names(a),)*
                }
            }
        }
    };

    (quote! {
        #ast_enum
        #(#node_structs)*
        #enum_impl
        #visitor
    }).into()
}