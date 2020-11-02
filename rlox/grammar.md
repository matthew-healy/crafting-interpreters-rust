# Lox Grammar Reference

```
program     -> declaration* ;

declaration -> varDecl | statement ;

varDecl     -> "var" IDENTIFIER ( "=" expression )? ";" ;

statement   -> exprStmt | printStmt | block ;
exprStmt    -> expression ";" ;
printStmt   -> "print" expression ";" ;
block       -> "{" declaration* "}" ;

expression  -> assignment ;
assignment  -> IDENTIFIER "=" assignment | equality ;
equality    -> ( ( "!=" | "==" ) comparison )* ;
comparison  -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term        -> factor ( ( "-" | "+" ) factor )* ;
factor      -> unary ( ( "/"  | "*" ) unary )* ;
unary       -> ( "!" | "-" -) unary | primary ;
primary     -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER ;
```