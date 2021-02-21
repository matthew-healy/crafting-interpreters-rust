use std::collections::HashMap;

use crate::{
    interpreter::Interpreter, 
    error::{Error, Result}, 
    expr::{self, Expr}, 
    stmt::{self, Stmt}, 
    token::Token
};

#[derive(Debug)]
enum VariableState {
    Declared,
    Defined,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum FunctionType {
    None,
    Function,
    Method,
    Init,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ClassType {
    None,
    Class,
}

pub struct Resolver<'a, W> {
    interpreter: &'a mut Interpreter<W>,
    scopes: Vec<HashMap<String, VariableState>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl <'a, W> Resolver<'a, W> {
    pub fn new(interpreter: &'a mut Interpreter<W>) -> Self {
        Resolver {
            interpreter,
            scopes: vec![HashMap::new()],
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve_stmts(&mut self, s: &[Stmt]) -> Result<()> {
        for stmt in s {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, s: &Stmt) -> Result<()> {
        s.accept(self)
    }

    fn resolve_expr(&mut self, e: &Expr) -> Result<()> {
        e.accept(self)
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, n: &Token) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&n.lexeme) {
                return Err(Error::static_analyzer(
                    n.clone(),
                    "A variable with this name already exists in this scope."
                ))
            }
            scope.insert(n.lexeme.clone(), VariableState::Declared);
        }
        Ok(())
    }

    fn define(&mut self, n: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(value) = scope.get_mut(&n.lexeme) {
                *value = VariableState::Defined;
            }
        }
    }

    fn resolve_local(&mut self, e: &Expr, n: &Token) {
        let index_and_scope = self.scopes.iter()
            .rev()
            .enumerate()
            .find(|s| s.1.contains_key(&n.lexeme));

        if let Some((idx, _)) = index_and_scope {
            self.interpreter.resolve(e, idx);
        }
    }

    fn resolve_function(&mut self, f: &stmt::Function, t: FunctionType) -> Result<()> {
        let enclosing_function = self.current_function;
        self.current_function = t;

        self.begin_scope();
        for param in f.params.iter() {
            self.declare(&param)?;
            self.define(&param);
        }
        self.resolve_stmts(&f.body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }
}

impl <'a, W> stmt::Visitor<Result<()>> for Resolver<'a, W> {
    fn visit_block_stmt(&mut self, b: &stmt::Block) -> Result<()> {
        self.begin_scope();
        self.resolve_stmts(&b.statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_class_stmt(&mut self, c: &stmt::Class) -> Result<()> {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;

        self.declare(&c.name)?;
        self.define(&c.name);

        if let Some(Expr::Variable(s)) = &c.superclass {
            if c.name.lexeme == s.name.lexeme {
                return Err(Error::static_analyzer(
                    s.name.clone(),
                    "A class may not inheret from itself."
                ))
            }
        }

        if let Some(superclass) = &c.superclass {
            self.resolve_expr(superclass)?;

            self.begin_scope();
            self.scopes.last_mut().and_then(|s|
                s.insert("super".into(), VariableState::Defined)
            );
        }

        self.begin_scope();
        self.scopes.last_mut().and_then(|s|
            s.insert("this".into(), VariableState::Defined)
        );

        for method in c.methods.iter() {
            let declaration = if method.name.lexeme == "init" {
                FunctionType::Init
            } else { FunctionType::Method };
            self.resolve_function(&method, declaration)?;
        }

        self.end_scope();

        if c.superclass.is_some() {
            self.end_scope();
        }

        self.current_class = enclosing_class;
        Ok(())
    }

    fn visit_expression_stmt(&mut self, e: &stmt::Expression) -> Result<()> {
        self.resolve_expr(&e.expression)
    }

    fn visit_function_stmt(&mut self, f: &stmt::Function) -> Result<()> {
        self.declare(&f.name)?;
        self.define(&f.name);
        self.resolve_function(&f, FunctionType::Function)?;
        Ok(())
    }

    fn visit_if_stmt(&mut self, i: &stmt::If) -> Result<()> {
        self.resolve_expr(&i.condition)?;
        self.resolve_stmt(&i.then_branch)?;
        if let Some(ref else_branch) = i.else_branch {
            self.resolve_stmt(else_branch)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, p: &stmt::Print) -> Result<()> {
        self.resolve_expr(&p.expression)
    }

    fn visit_return_stmt(&mut self, r: &stmt::Return) -> Result<()> {
        match self.current_function {
            FunctionType::None => Err(Error::static_analyzer(
                r.keyword.clone(),
                "Can't return from top-level code."
            )),
            FunctionType::Init if r.value.is_some() => Err(Error::static_analyzer(
                r.keyword.clone(),
                "Cannot return a value from an initialiser."
            )),
            _ => r.value.as_ref()
                    .map(|v| self.resolve_expr(v))
                    .unwrap_or(Ok(()))
        }
    }

    fn visit_var_stmt(&mut self, v: &stmt::Var) -> Result<()> {
        self.declare(&v.name)?;
        if let Some(ref i) = v.initializer {
            self.resolve_expr(i)?;
        }
        self.define(&v.name);
        Ok(())
    }

    fn visit_while_stmt(&mut self, w: &stmt::While) -> Result<()> {
        self.resolve_expr(&w.condition)?;
        self.resolve_stmt(&w.body)
    }
}

impl <'a, W> expr::Visitor<Result<()>> for Resolver<'a, W> {
    fn visit_assign_expr(&mut self, a: &expr::Assign) -> Result<()> {
        self.resolve_expr(&a.value)?;
        self.resolve_local(&Expr::Assign(a.clone()), &a.name);
        Ok(())
    }

    fn visit_binary_expr(&mut self, e: &expr::Binary) -> Result<()> {
        self.resolve_expr(&e.left)?;
        self.resolve_expr(&e.right)
    }

    fn visit_call_expr(&mut self, e: &expr::Call) -> Result<()> {
        self.resolve_expr(&e.callee)?;

        for argument in &e.arguments {
            self.resolve_expr(&argument)?;
        }

        Ok(())
    }

    fn visit_get_expr(&mut self, g: &expr::Get) -> Result<()> {
        self.resolve_expr(&g.object)
    }

    fn visit_grouping_expr(&mut self, e: &expr::Grouping) -> Result<()> {
        self.resolve_expr(&e.expression)
    }

    fn visit_literal_expr(&mut self, _e: &expr::Literal) -> Result<()> {
        Ok(())
    }

    fn visit_logical_expr(&mut self, e: &expr::Logical) -> Result<()> {
        self.resolve_expr(&e.left)?;
        self.resolve_expr(&e.right)
    }

    fn visit_set_expr(&mut self, e: &expr::Set) -> Result<()> {
        self.resolve_expr(&e.value)?;
        self.resolve_expr(&e.object)
    }

    fn visit_super_expr(&mut self, e: &expr::Super) -> Result<()> {
        self.resolve_local(&Expr::Super(e.clone()), &e.keyword);
        Ok(())
    }

    fn visit_this_expr(&mut self, e: &expr::This) -> Result<()> {
        match self.current_class {
            ClassType::None => Err(Error::syntactic(
                e.keyword.clone(),
                "Can't use 'this' outside of a class."
            )),
            _ => Ok(self.resolve_local(&Expr::This(e.clone()), &e.keyword))
        }
    }

    fn visit_unary_expr(&mut self, e: &expr::Unary) -> Result<()> {
        self.resolve_expr(&e.right)
    }

    fn visit_variable_expr(&mut self, e: &expr::Variable) -> Result<()> {
        match self.scopes.last().and_then(|s| s.get(&e.name.lexeme)) {
            Some(VariableState::Declared) => {
                Err(Error::static_analyzer(
                    e.name.clone(), 
                    "Can't read local variable in its own initializer."
                ))
            },
            _ => {
                // jlox uses inheritance for AST nodes, but we have an enum so
                // we need to reconstruct the Expr case to resolve the variable.
                self.resolve_local(&Expr::Variable(e.clone()), &e.name);
                Ok(())
            }
        }
    }
}