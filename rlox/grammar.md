# Lox Grammar Reference

```
program     -> declaration* ;

declaration -> funDecl | varDecl | statement ;
funDecl     -> "fun" function ;
function    -> IDENTIFIER "(" parameters? ")" block ;
parameters  -> IDENTIFIER ( "," IDENTIFIER )* ;
varDecl     -> "var" IDENTIFIER ( "=" expression )? ";" ;

statement   -> exprStmt | forStmt | ifStmt | printStmt | whileStmt | block ;
exprStmt    -> expression ";" ;
forStmt     -> "for" "(" ( varDecl | exprStmt | ";" ) expression? ";" expression? ")" statement ;
ifStmt      -> "if" "(" expression ")" statement ( "else" statement )? ;
printStmt   -> "print" expression ";" ;
whileStmt   -> "while" "(" expression ")" statement ;
block       -> "{" declaration* "}" ;

expression  -> assignment ;
assignment  -> IDENTIFIER "=" assignment | logic_or ;
logic_or    -> logic_and ( "or" logic_and )* ;
logic_and   -> equality ( "and" equality )* ;
equality    -> ( ( "!=" | "==" ) comparison )* ;
comparison  -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term        -> factor ( ( "-" | "+" ) factor )* ;
factor      -> unary ( ( "/"  | "*" ) unary )* ;
unary       -> ( "!" | "-" -) unary | primary ;
call        -> primary ( "(" arguments? ")" )* ;
arguments   -> expression ( "," expression )* ;
primary     -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER ;
```