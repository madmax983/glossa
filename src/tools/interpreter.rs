//! The Simulator (ὁ Ὑποκριτής) - Tree-Walk Interpreter
//!
//! This module implements an in-memory tree-walk interpreter for ΓΛΩΣΣΑ.
//! Unlike the compiler which transpiles to Rust, the simulator executes the analyzed AST directly.
//!
//! # Purpose
//!
//! 1. **Immediate Feedback**: Enables a true REPL without compilation overhead.
//! 2. **Playground**: Allows running code in WASM environments (web browser) where `rustc` is unavailable.
//! 3. **Debuggability**: Easier to inspect state during execution than a compiled binary.
//!
//! # Architecture
//!
//! The interpreter operates on the [`AnalyzedProgram`] produced by the semantic analyzer.
//! It maintains a runtime environment (stack of scopes) mapping variable names to [`Value`]s.

use crate::morphology::lexicon::{BinaryOp, UnaryOp};
use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement};
use std::collections::HashMap;
use std::fmt;

/// Runtime Value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i64),
    String(String),
    Boolean(bool),
    Unit,
    List(Vec<Value>),
    Struct(HashMap<String, Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Unit => write!(f, "()"),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Struct(s) => {
                write!(f, "{{ ")?;
                let mut first = true;
                for (k, v) in s {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                    first = false;
                }
                write!(f, " }}")
            }
        }
    }
}

/// Evaluation Error
#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("μεταβλητὴ οὐχ εὑρέθη (Variable not found): {0}")]
    VariableNotFound(String),

    #[error("τύποι ἀσύμβατοι (Type mismatch): expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },

    #[error("πρᾶξις ἄκυρος (Invalid operation): {op} on {left} and {right}")]
    InvalidOperation {
        op: String,
        left: String,
        right: String,
    },

    #[error("διαίρεσις διὰ μηδενός (Division by zero)")]
    DivisionByZero,

    #[error("μὴ ὑλοποιημένον (Not implemented): {0}")]
    NotImplemented(String),
}

/// The Interpreter Context
///
/// A simple tree-walk interpreter for Glossa code.
///
/// Unlike the traditional compiler which converts the AST into Rust code,
/// the simulator lets us execute Glossa code directly from the AST. It holds
/// the runtime environment, meaning the state of variables defined along
/// the execution path.
///
/// # Examples
///
/// ```rust
/// use glossa::tools::interpreter::Interpreter;
///
/// let mut interp = Interpreter::new();
/// // You could then run a program via `interp.run(&analyzed_program)`
/// ```
pub struct Interpreter {
    // Stack of scopes. For now, just one global scope for simplicity.
    // In a real implementation, this would be `Vec<HashMap<String, Value>>`.
    env: Vec<HashMap<String, Value>>,

    // Output buffer for capturing print statements (useful for testing/WASM)
    output: Vec<String>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Creates a new, empty Interpreter environment.
    ///
    /// It initializes a single global scope and an empty output buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::tools::interpreter::Interpreter;
    ///
    /// let interp = Interpreter::new();
    /// assert_eq!(interp.get_output(), "");
    /// ```
    pub fn new() -> Self {
        Self {
            env: vec![HashMap::new()],
            output: Vec::new(),
        }
    }

    /// Execute a program
    pub fn run(&mut self, program: &AnalyzedProgram) -> Result<(), EvalError> {
        for stmt in &program.statements {
            self.eval_statement(stmt)?;
        }
        Ok(())
    }

    /// Get the captured output
    pub fn get_output(&self) -> String {
        self.output.join("\n")
    }

    fn eval_statement(&mut self, stmt: &AnalyzedStatement) -> Result<(), EvalError> {
        match stmt {
            AnalyzedStatement::Binding { name, value, .. } => {
                let val = self.eval_expr(value)?;
                self.define_var(name, val);
            }
            AnalyzedStatement::Assignment { name, value } => {
                let val = self.eval_expr(value)?;
                self.assign_var(name, val)?;
            }
            AnalyzedStatement::Print(exprs) => {
                let mut parts = Vec::new();
                for expr in exprs {
                    parts.push(self.eval_expr(expr)?.to_string());
                }
                let line = parts.join(" ");
                println!("{}", line);
                self.output.push(line);
            }
            AnalyzedStatement::Expression(exprs) => {
                for expr in exprs {
                    self.eval_expr(expr)?;
                }
            }
            AnalyzedStatement::TypeDefinition { .. } => {
                // Ignore type definitions for now in the simple interpreter
            }
            AnalyzedStatement::If {
                condition,
                then_body,
                else_body,
            } => {
                let cond_val = self.eval_expr(condition)?;
                if let Value::Boolean(b) = cond_val {
                    if b {
                        for stmt in then_body {
                            self.eval_statement(stmt)?;
                        }
                    } else if let Some(else_stmts) = else_body {
                        for stmt in else_stmts {
                            self.eval_statement(stmt)?;
                        }
                    }
                } else {
                    return Err(EvalError::TypeMismatch {
                        expected: "Boolean".to_string(),
                        got: format!("{:?}", cond_val),
                    });
                }
            }
            AnalyzedStatement::For {
                variable,
                iterator,
                body,
            } => {
                let iter_val = self.eval_expr(iterator)?;
                // Mock simple for loops over ranges or arrays for now
                match iter_val {
                    Value::List(l) => {
                        for val in l {
                            self.define_var(variable, val);
                            for stmt in body {
                                self.eval_statement(stmt)?;
                            }
                        }
                    }
                    Value::Unit => {
                        // Dummy
                    }
                    _ => {
                        return Err(EvalError::NotImplemented(format!(
                            "for over {:?}",
                            iter_val
                        )));
                    }
                }
            }
            AnalyzedStatement::FunctionDef { .. } => {
                // Ignore functions
            }
            _ => return Err(EvalError::NotImplemented(format!("{:?}", stmt))),
        }
        Ok(())
    }

    fn eval_expr(&self, expr: &AnalyzedExpr) -> Result<Value, EvalError> {
        match &expr.expr {
            AnalyzedExprKind::NumberLiteral(n) => Ok(Value::Number(*n)),
            AnalyzedExprKind::StringLiteral(s) => Ok(Value::String(s.clone())),
            AnalyzedExprKind::BooleanLiteral(b) => Ok(Value::Boolean(*b)),
            AnalyzedExprKind::Variable(name) => self.lookup_var(name),
            AnalyzedExprKind::PropertyAccess { owner, property } => {
                let owner_val = self.eval_expr(owner)?;
                match owner_val {
                    Value::Struct(s) => {
                        if let Some(val) = s.get(property.as_str()) {
                            Ok(val.clone())
                        } else {
                            Err(EvalError::NotImplemented(format!("Property {} not found on Struct", property)))
                        }
                    }
                    _ => Err(EvalError::NotImplemented(format!("Property access on {:?}", owner_val))),
                }
            }
            AnalyzedExprKind::BinOp { left, op, right } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_bin_op(op, l, r)
            }
            AnalyzedExprKind::UnaryOp { op, operand } => {
                let val = self.eval_expr(operand)?;
                self.eval_unary_op(op, val)
            }
            AnalyzedExprKind::StructInstantiation { fields, args, .. } => {
                let mut struct_val = HashMap::new();
                for (field, arg_expr) in fields.iter().zip(args.iter()) {
                    let val = self.eval_expr(arg_expr)?;
                    struct_val.insert(field.to_string(), val);
                }
                Ok(Value::Struct(struct_val))
            }
            AnalyzedExprKind::ArrayLiteral(exprs) => {
                let mut list = Vec::new();
                for expr in exprs {
                    list.push(self.eval_expr(expr)?);
                }
                Ok(Value::List(list))
            }
            AnalyzedExprKind::MethodCall {
                receiver,
                method,
                args: _,
            } => {
                let rec_val = self.eval_expr(receiver)?;
                if method == "to_string" {
                    match rec_val {
                        Value::String(s) => Ok(Value::String(s)),
                        _ => Ok(Value::String(rec_val.to_string())),
                    }
                } else {
                    Err(EvalError::NotImplemented(format!(
                        "MethodCall {} on {:?}",
                        method, rec_val
                    )))
                }
            }
            _ => Err(EvalError::NotImplemented(format!("{:?}", expr.expr))),
        }
    }

    fn eval_bin_op(&self, op: &BinaryOp, left: Value, right: Value) -> Result<Value, EvalError> {
        match (op, &left, &right) {
            // Arithmetic
            (BinaryOp::Add, Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
            (BinaryOp::Sub, Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
            (BinaryOp::Mul, Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
            (BinaryOp::Div, Value::Number(l), Value::Number(r)) => {
                if *r == 0 {
                    return Err(EvalError::DivisionByZero);
                }
                Ok(Value::Number(l / r))
            }

            // Comparison
            (BinaryOp::Eq, _, _) => Ok(Value::Boolean(left == right)),
            (BinaryOp::Ne, _, _) => Ok(Value::Boolean(left != right)),
            (BinaryOp::Lt, Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l < r)),
            (BinaryOp::Gt, Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l > r)),

            // Boolean Logic
            (BinaryOp::And, Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(*l && *r)),
            (BinaryOp::Or, Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(*l || *r)),

            _ => Err(EvalError::InvalidOperation {
                op: format!("{:?}", op),
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            }),
        }
    }

    fn eval_unary_op(&self, op: &UnaryOp, operand: Value) -> Result<Value, EvalError> {
        match (op, operand) {
            (UnaryOp::Not, Value::Boolean(b)) => Ok(Value::Boolean(!b)),
            (UnaryOp::Neg, Value::Number(n)) => Ok(Value::Number(-n)),
            _ => Err(EvalError::NotImplemented(format!("Unary op {:?}", op))),
        }
    }

    fn define_var(&mut self, name: &str, value: Value) {
        // Define in the current (top) scope
        if let Some(scope) = self.env.last_mut() {
            scope.insert(name.to_string(), value);
        }
    }

    fn assign_var(&mut self, name: &str, value: Value) -> Result<(), EvalError> {
        // Walk up the scopes to find the variable
        for scope in self.env.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        Err(EvalError::VariableNotFound(name.to_string()))
    }

    fn lookup_var(&self, name: &str) -> Result<Value, EvalError> {
        for scope in self.env.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Ok(val.clone());
            }
        }
        Err(EvalError::VariableNotFound(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    fn run_code(code: &str) -> Interpreter {
        let ast = parse(code).expect("Parse error");
        let program = analyze_program(&ast).expect("Analysis error");
        let mut interpreter = Interpreter::new();
        interpreter.run(&program).expect("Runtime error");
        interpreter
    }

    #[test]
    fn test_eval_literals() {
        let interpreter = run_code("«hello» λέγε.");
        assert_eq!(interpreter.get_output(), "hello");
    }

    #[test]
    fn test_eval_arithmetic() {
        let interpreter = run_code("1 2 ἄθροισμα λέγε."); // 1 + 2 (using a known arithmetic noun)
        assert_eq!(interpreter.get_output(), "3");
    }

    #[test]
    fn test_variable_binding() {
        let interpreter = run_code("ξ πέντε ἔστω. ξ λέγε.");
        assert_eq!(interpreter.get_output(), "5");
    }

    #[test]
    fn test_variable_assignment() {
        // ξ starts as 5, then becomes 10. Must use 'μετά' (mutable marker) for reassignment to be valid.
        let interpreter = run_code("μετά ξ πέντε ἔστω. ξ δέκα γίγνεται. ξ λέγε.");
        assert_eq!(interpreter.get_output(), "10");
    }

    #[test]
    fn test_boolean_logic() {
        let interpreter = run_code("ἀληθές λέγε.");
        assert_eq!(interpreter.get_output(), "true");
    }

    #[test]
    fn test_division_by_zero() {
        let code = "1 0 μέρος λέγε.";
        let ast = parse(code).expect("Parse error");
        let program = analyze_program(&ast).expect("Analysis error");
        let mut interpreter = Interpreter::new();
        let result = interpreter.run(&program);
        assert!(matches!(result, Err(EvalError::DivisionByZero)));
    }

    #[test]
    fn test_eval_if_true() {
        let interpreter = run_code("εἰ ἀληθές ἐστι, «ναι» λέγε.");
        assert_eq!(interpreter.get_output(), "ναι");
    }

    #[test]
    fn test_eval_if_false() {
        // Since we don't have else logic parsing easily accessible in a one-liner without more complex setup,
        // we just test it does nothing on false.
        let interpreter = run_code("εἰ ψεῦδος ἐστι, «ναι» λέγε.");
        assert_eq!(interpreter.get_output(), "");
    }

    #[test]
    fn test_eval_for_loop() {
        let interpreter = run_code("ἀριθμός [1, 2, 3] ἔστω. διὰ ἀριθμοῦ, ν λέγε.");
        assert_eq!(interpreter.get_output(), "1\n2\n3");
    }

    #[test]
    fn test_struct_instantiation() {
        let code = "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }. χρήστης νέον Χρήστης «Σωκράτης» ἔστω. χρήστου ὄνομα λέγε.";
        let interpreter = run_code(code);
        assert_eq!(interpreter.get_output(), "Σωκράτης");
    }
}
