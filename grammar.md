# Lox Grammar Reference

```
expression -> equality;
equality   -> ( ( "!=" | "==" ) comparison )*;
comparison -> term ( ( ">" | ">=" | "<" | "<=" ) term )*;
term       -> factor ( ( "-" | "+" ) factor )*;
factor     -> unary ( ( "/"  | "*" ) unary )*;
unary      -> ( "!" | "-" -) unary | primary;
primary    -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")";
```