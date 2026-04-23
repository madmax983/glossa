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
use rustc_hash::FxHashMap;
use std::fmt;

/// The fundamental unit of data in the Simulator's runtime environment.
///
/// In a statically typed system, variables have types known at compile time.
/// However, during the interpretation phase (or simulation), we need a dynamic container
/// to hold the actual runtime data. The [`Value`] enum acts as this universal container,
/// allowing the simulator to safely pass data between variables, functions, and operations
/// without relying on Rust's type system to manage the simulated user's memory.
///
/// Every [`Value`] can be displayed natively as text, which powers the simulator's output.
///
/// ## Examples
///
/// ```rust
/// use glossa::tools::interpreter::Value;
///
/// // Create a numerical truth
/// let count = Value::Number(42);
///
/// // Create a boolean truth
/// let is_valid = Value::Boolean(true);
///
/// assert_eq!(count.to_string(), "42");
/// assert_eq!(is_valid.to_string(), "true");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A signed 64-bit integer.
    Number(i64),
    /// A UTF-8 string dynamically allocated on the heap.
    String(String),
    /// A simple boolean truth state (`true` or `false`).
    Boolean(bool),
    /// An empty vessel signifying the absence of meaningful data.
    Unit,
    // Future: List(Vec<Value>), Struct(HashMap<String, Value>), etc.
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Unit => write!(f, "()"),
        }
    }
}

/// Represents the inevitable failures of a simulated runtime environment.
///
/// Unlike compiler errors (which happen when analyzing the AST and typing code),
/// [`EvalError`] exists to catch dynamic failures that only manifest during program execution.
/// A user's code might type-check perfectly, but attempting to mutate a nonexistent variable,
/// divide by zero, or push numeric limits at runtime will trigger these failures.
///
/// Because the simulator mimics the compiler, these errors also speak Ancient Greek natively.
///
/// ## Examples
///
/// ```rust
/// use glossa::tools::interpreter::EvalError;
///
/// let missing_var = EvalError::VariableNotFound("x".into());
/// assert!(missing_var.to_string().contains("μεταβλητὴ"));
/// ```
#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    /// Raised when a variable is referenced before it is defined via `ἔστω`.
    #[error("μεταβλητὴ οὐχ εὑρέθη (Variable not found): {0}")]
    VariableNotFound(String),

    /// Raised when a dynamic type assertion fails.
    #[error("τύποι ἀσύμβατοι (Type mismatch): expected {expected}, got {got}")]
    TypeMismatch {
        /// The expected underlying representation.
        expected: String,
        /// The actual provided representation.
        got: String,
    },

    /// Triggered when an operator receives two incompatible [`Value`]s (like adding a `String` to a `Boolean`).
    #[error("πρᾶξις ἄκυρος (Invalid operation): {op} on {left} and {right}")]
    InvalidOperation {
        /// The mathematical or logical operator that failed.
        op: String,
        /// The left-hand side of the broken equation.
        left: String,
        /// The right-hand side of the broken equation.
        right: String,
    },

    /// Triggered to prevent a panic when a simulated division operation meets zero.
    #[error("διαίρεσις διὰ μηδενός (Division by zero)")]
    DivisionByZero,

    /// Triggered when mathematical operations exceed the bounds of `i64`.
    #[error("ἀριθμητικὴ ὑπερχείλισις (Arithmetic overflow)")]
    ArithmeticOverflow,

    /// A fallback state indicating the simulation engine does not yet support a specific AST node.
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
    // In a real implementation, this would be `Vec<FxHashMap<String, Value>>`.
    env: Vec<FxHashMap<String, Value>>,

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
            env: vec![FxHashMap::default()],
            output: Vec::new(),
        }
    }

    /// Execute a program
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::tools::interpreter::Interpreter;
    /// use glossa::parser::parse;
    /// use glossa::semantic::analyze_program;
    ///
    /// let code = "ξ πέντε ἔστω. ξ λέγε.";
    /// let ast = parse(code).unwrap();
    /// let program = analyze_program(&ast).unwrap();
    ///
    /// let mut interp = Interpreter::new();
    /// interp.run(&program).unwrap();
    /// assert_eq!(interp.get_output(), "5");
    /// ```
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
                use std::fmt::Write;
                // ⚡ Bolt Optimization: Removed intermediate `.collect::<Vec<_>>()` allocation and `.join(" ")`.
                let mut line = String::with_capacity(exprs.len() * 16);
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        line.push(' ');
                    }
                    let _ = write!(&mut line, "{}", self.eval_expr(expr)?);
                }
                println!("{}", line);
                self.output.push(line);
            }
            AnalyzedStatement::Expression(exprs) => {
                for expr in exprs {
                    self.eval_expr(expr)?;
                }
            }
            _ => return Err(EvalError::NotImplemented(format!("{:?}", stmt))),
        }
        Ok(())
    }

    /// Evaluates a given expression into a value.
    fn eval_expr(&self, expr: &AnalyzedExpr) -> Result<Value, EvalError> {
        match &expr.expr {
            AnalyzedExprKind::NumberLiteral(n) => Ok(Value::Number(*n)),
            AnalyzedExprKind::StringLiteral(s) => Ok(Value::String(s.clone())),
            AnalyzedExprKind::BooleanLiteral(b) => Ok(Value::Boolean(*b)),
            AnalyzedExprKind::Variable(name) => self.lookup_var(name),
            AnalyzedExprKind::BinOp { left, op, right } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_bin_op(op, l, r)
            }
            AnalyzedExprKind::UnaryOp { op, operand } => {
                let val = self.eval_expr(operand)?;
                self.eval_unary_op(op, val)
            }
            _ => Err(EvalError::NotImplemented(format!("{:?}", expr.expr))),
        }
    }

    fn eval_bin_op(&self, op: &BinaryOp, left: Value, right: Value) -> Result<Value, EvalError> {
        match (op, &left, &right) {
            // Arithmetic
            (BinaryOp::Add, Value::Number(l), Value::Number(r)) => l
                .checked_add(*r)
                .map(Value::Number)
                .ok_or(EvalError::ArithmeticOverflow),
            (BinaryOp::Sub, Value::Number(l), Value::Number(r)) => l
                .checked_sub(*r)
                .map(Value::Number)
                .ok_or(EvalError::ArithmeticOverflow),
            (BinaryOp::Mul, Value::Number(l), Value::Number(r)) => l
                .checked_mul(*r)
                .map(Value::Number)
                .ok_or(EvalError::ArithmeticOverflow),
            (BinaryOp::Div, Value::Number(l), Value::Number(r)) => {
                if *r == 0 {
                    return Err(EvalError::DivisionByZero);
                }
                l.checked_div(*r)
                    .map(Value::Number)
                    .ok_or(EvalError::ArithmeticOverflow)
            }
            (BinaryOp::Mod, Value::Number(l), Value::Number(r)) => {
                if *r == 0 {
                    return Err(EvalError::DivisionByZero);
                }
                l.checked_rem(*r)
                    .map(Value::Number)
                    .ok_or(EvalError::ArithmeticOverflow)
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
            (UnaryOp::Neg, Value::Number(n)) => n
                .checked_neg()
                .map(Value::Number)
                .ok_or(EvalError::ArithmeticOverflow),
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
    fn test_modulo_by_zero() {
        let code = "1 0 ὑπόλοιπον λέγε.";
        let ast = parse(code).expect("Parse error");
        let program = analyze_program(&ast).expect("Analysis error");
        let mut interpreter = Interpreter::new();
        let result = interpreter.run(&program);
        assert!(matches!(result, Err(EvalError::DivisionByZero)));
    }

    #[test]
    fn test_default_impl() {
        let interp = Interpreter::default();
        assert_eq!(interp.env.len(), 1);
        assert!(interp.output.is_empty());
    }

    #[test]
    fn test_value_display() {
        assert_eq!(Value::Unit.to_string(), "()");
        assert_eq!(Value::Number(42).to_string(), "42");
        assert_eq!(Value::Boolean(true).to_string(), "true");
        assert_eq!(Value::String("test".to_string()).to_string(), "test");
    }

    #[test]
    fn test_eval_expression_statement() {
        // Just evaluating an expression without printing or binding
        let stmt = AnalyzedStatement::Expression(vec![AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(42),
            glossa_type: crate::semantic::GlossaType::Number,
        }]);
        let program = AnalyzedProgram {
            statements: vec![stmt],
            scope: crate::semantic::Scope::new(),
        };
        let mut interpreter = Interpreter::new();
        interpreter.run(&program).expect("Runtime error");
        assert!(interpreter.get_output().is_empty());
    }

    #[test]
    fn test_not_implemented_statement() {
        let stmt = AnalyzedStatement::Return { value: None };
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_statement(&stmt);
        assert!(matches!(result, Err(EvalError::NotImplemented(_))));
    }

    #[test]
    fn test_not_implemented_expression() {
        let expr2 = AnalyzedExpr {
            expr: AnalyzedExprKind::StructInstantiation {
                type_name: "TestStruct".into(),
                fields: vec![],
                args: vec![],
            },
            glossa_type: crate::semantic::GlossaType::Unknown,
        };
        let interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&expr2);
        assert!(matches!(result, Err(EvalError::NotImplemented(_))));
    }

    #[test]
    fn test_eval_unary_op() {
        let interpreter = Interpreter::new();

        let not_true = interpreter
            .eval_unary_op(&UnaryOp::Not, Value::Boolean(true))
            .unwrap();
        assert_eq!(not_true, Value::Boolean(false));

        let neg_five = interpreter
            .eval_unary_op(&UnaryOp::Neg, Value::Number(5))
            .unwrap();
        assert_eq!(neg_five, Value::Number(-5));

        let invalid_unary = interpreter.eval_unary_op(&UnaryOp::Not, Value::Number(1));
        assert!(matches!(invalid_unary, Err(EvalError::NotImplemented(_))));

        // Test the eval_expr wrapper around unary ops
        let unary_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::UnaryOp {
                op: UnaryOp::Not,
                operand: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: crate::semantic::GlossaType::Boolean,
                }),
            },
            glossa_type: crate::semantic::GlossaType::Boolean,
        };
        let unary_eval = interpreter.eval_expr(&unary_expr).unwrap();
        assert_eq!(unary_eval, Value::Boolean(false));
    }

    #[test]
    fn test_eval_binary_op() {
        let interpreter = Interpreter::new();

        // Sub
        assert_eq!(
            interpreter
                .eval_bin_op(&BinaryOp::Sub, Value::Number(10), Value::Number(4))
                .unwrap(),
            Value::Number(6)
        );

        // Mul
        assert_eq!(
            interpreter
                .eval_bin_op(&BinaryOp::Mul, Value::Number(3), Value::Number(4))
                .unwrap(),
            Value::Number(12)
        );

        // Div (Success)
        assert_eq!(
            interpreter
                .eval_bin_op(&BinaryOp::Div, Value::Number(12), Value::Number(3))
                .unwrap(),
            Value::Number(4)
        );

        // Mod (Success)
        assert_eq!(
            interpreter
                .eval_bin_op(&BinaryOp::Mod, Value::Number(12), Value::Number(5))
                .unwrap(),
            Value::Number(2)
        );

        // Eq & Ne (Strings, to hit the general case)
        assert_eq!(
            interpreter
                .eval_bin_op(
                    &BinaryOp::Eq,
                    Value::String("a".into()),
                    Value::String("a".into())
                )
                .unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            interpreter
                .eval_bin_op(
                    &BinaryOp::Ne,
                    Value::String("a".into()),
                    Value::String("b".into())
                )
                .unwrap(),
            Value::Boolean(true)
        );

        // Lt & Gt
        assert_eq!(
            interpreter
                .eval_bin_op(&BinaryOp::Lt, Value::Number(3), Value::Number(5))
                .unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            interpreter
                .eval_bin_op(&BinaryOp::Gt, Value::Number(5), Value::Number(3))
                .unwrap(),
            Value::Boolean(true)
        );

        // And & Or
        assert_eq!(
            interpreter
                .eval_bin_op(&BinaryOp::And, Value::Boolean(true), Value::Boolean(false))
                .unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            interpreter
                .eval_bin_op(&BinaryOp::Or, Value::Boolean(false), Value::Boolean(true))
                .unwrap(),
            Value::Boolean(true)
        );

        // Invalid Operation
        let invalid = interpreter.eval_bin_op(
            &BinaryOp::Add,
            Value::Number(1),
            Value::String("foo".into()),
        );
        assert!(matches!(invalid, Err(EvalError::InvalidOperation { .. })));
    }

    #[test]
    fn test_variable_not_found() {
        let mut interpreter = Interpreter::new();

        // Lookup
        assert!(matches!(
            interpreter.lookup_var("x"),
            Err(EvalError::VariableNotFound(_))
        ));

        // Assign
        assert!(matches!(
            interpreter.assign_var("x", Value::Number(1)),
            Err(EvalError::VariableNotFound(_))
        ));
    }
}
