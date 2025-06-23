use crate::expression::Expression;
use crate::interpreter::Interpreter;
use crate::stmt::{BlockStmt, Stmt};
use crate::tokens::Token;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ResolverError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
enum FunctionType {
    None,
    Function,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}
impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
        }
    }

    pub fn resolve_stmts(&mut self, statements: &Vec<Stmt>) -> Result<(), ResolverError> {
        for statement in statements {
            self.resolve_stmt(statement)?;
        }
        Ok(())
    }

    pub fn resolve_stmt(&mut self, statement: &Stmt) -> Result<(), ResolverError> {
        match statement {
            Stmt::Expression(expr) => self.resolve_expr(&expr.expression),
            Stmt::Print(expr) => self.resolve_expr(&expr.expression),
            Stmt::Var(expr) => {
                self.declare(&expr.name)?;
                if let Some(init) = &expr.initializer {
                    self.resolve_expr(&init)?;
                }
                self.define(&expr.name)?;
                Ok(())
            }
            Stmt::Block(expr) => self.resolve_block(&expr),
            Stmt::If(expr) => {
                self.resolve_expr(&expr.condition)?;
                self.resolve_stmt(&expr.then_branch)?;
                if let Some(else_branch) = &expr.else_branch {
                    self.resolve_stmt(else_branch.as_ref())?;
                }
                Ok(())
            }
            Stmt::While(expr) => {
                self.resolve_expr(&expr.condition)?;
                self.resolve_stmt(expr.body.as_ref())?;
                Ok(())
            }
            Stmt::Return(expr) => {
                if self.current_function == FunctionType::None {
                    let name = expr.keyword.lexeme.clone();
                    let line = expr.keyword.line;
                    return Err(ResolverError {
                        message: format!(
                            "[line {}] Error at '{}': {}",
                            line, name, "Can't return from top-level code."
                        ),
                    });
                }
                if let Some(val) = &expr.value {
                    self.resolve_expr(val.as_ref())?;
                }
                Ok(())
            }
            Stmt::Function(expr) => {
                self.declare(&expr.name)?;
                self.define(&expr.name)?;
                self.resolve_function(&expr.params, &expr.body, FunctionType::Function)?;
                Ok(())
            }
            Stmt::Class(expr) => {
                self.declare(&expr.name)?;
                self.define(&expr.name)?;
                Ok(())
            }
        }
    }

    fn resolve_block(&mut self, block: &BlockStmt) -> Result<(), ResolverError> {
        self.begin_scope();
        self.resolve_stmts(&block.statements)?;
        self.end_scope();
        Ok(())
    }

    fn resolve_function(
        &mut self,
        params: &Vec<Token>,
        body: &Vec<Stmt>,
        function_type: FunctionType,
    ) -> Result<(), ResolverError> {
        let enclosing_function = self.current_function.clone();
        self.current_function = function_type;
        self.begin_scope();
        for param in params {
            self.declare(param)?;
            self.define(param)?;
        }
        self.resolve_stmts(body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &Expression) -> Result<(), ResolverError> {
        match expr {
            Expression::Variable(var) => {
                if !self.scopes.is_empty()
                    && self.scopes.last().unwrap().get(&var.name.lexeme) == Some(&false)
                {
                    let name = var.name.lexeme.clone();
                    let line = var.name.line;
                    return Err(ResolverError {
                        message: format!(
                            "[line {}] Error at '{}': {}",
                            line, name, "Can't read local variable in its own initializer."
                        ),
                    });
                }
                self.resolve_local(expr, &var.name)?;
                Ok(())
            }
            Expression::Assign(assign) => {
                self.resolve_expr(assign.value.as_ref())?;
                self.resolve_local(expr, &assign.name)?;
                Ok(())
            }
            Expression::Binary(binary) => {
                self.resolve_expr(binary.left.as_ref())?;
                self.resolve_expr(binary.right.as_ref())?;
                Ok(())
            }
            Expression::Call(call) => {
                self.resolve_expr(call.callee.as_ref())?;
                for arg in call.arguments.iter() {
                    self.resolve_expr(arg)?;
                }
                Ok(())
            }
            Expression::Get(get) => {
                self.resolve_expr(get.object.as_ref())?;
                Ok(())
            }
            Expression::Grouping(group) => {
                self.resolve_expr(&group.expression)?;
                Ok(())
            }
            Expression::Literal(_) => Ok(()),
            Expression::Logical(logical) => {
                self.resolve_expr(logical.left.as_ref())?;
                self.resolve_expr(logical.right.as_ref())?;
                Ok(())
            }
            Expression::Set(set) => {
                self.resolve_expr(set.value.as_ref())?;
                self.resolve_expr(set.object.as_ref())?;
                Ok(())
            }
            Expression::Unary(unary) => {
                self.resolve_expr(unary.right.as_ref())?;
                Ok(())
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn end_scope(&mut self) {
        self.scopes.pop();
    }
    fn declare(&mut self, token: &Token) -> Result<(), ResolverError> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&token.lexeme) {
                let name = token.lexeme.clone();
                let line = token.line;
                return Err(ResolverError {
                    message: format!(
                        "[line {}] Error at '{}': {}",
                        line, name, "Already a variable with this name in this scope."
                    ),
                });
            }
            scope.insert(token.lexeme.clone(), false);
        }
        Ok(())
    }
    fn define(&mut self, token: &Token) -> Result<(), ResolverError> {
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(defined) = scope.get_mut(&token.lexeme) {
                *defined = true;
            } else {
                let name = token.lexeme.clone();
                let line = token.line;
                return Err(ResolverError {
                    message: format!(
                        "[line {}] Error at '{}': {}",
                        line, name, "Variable not declared in this scope."
                    ),
                });
            }
        }
        Ok(())
    }
    fn resolve_local(&mut self, expr: &Expression, name: &Token) -> Result<(), ResolverError> {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, self.scopes.len() - 1 - i);
                break;
            }
        }
        Ok(())
    }
}
