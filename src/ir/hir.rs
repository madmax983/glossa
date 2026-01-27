//! High-level Intermediate Representation
//!
//! A simplified IR that maps closely to Rust constructs.

use crate::semantic::{AnalyzedProgram, AnalyzedStatement, StatementKind, AnalyzedExpr, AnalyzedExprKind};

/// A HIR program ready for code generation
#[derive(Debug, Clone)]
pub struct HirProgram {
    pub statements: Vec<HirStatement>,
}

/// A HIR statement
#[derive(Debug, Clone)]
pub enum HirStatement {
    /// let name = value;
    Let {
        name: String,
        value: HirExpr,
        mutable: bool,
    },

    /// println!(...);
    Print {
        args: Vec<HirExpr>,
    },

    /// expression;
    Expr(HirExpr),

    /// if condition { then_body } else { else_body }
    If {
        condition: HirExpr,
        then_body: Vec<HirStatement>,
        else_body: Option<Vec<HirStatement>>,
    },

    /// while condition { body }
    While {
        condition: HirExpr,
        body: Vec<HirStatement>,
    },

    /// for var in iterator { body }
    For {
        variable: String,
        iterator: HirExpr,
        body: Vec<HirStatement>,
    },

    /// match scrutinee { arms }
    Match {
        scrutinee: HirExpr,
        arms: Vec<(HirExpr, Vec<HirStatement>)>,
    },

    /// break;
    Break,

    /// continue;
    Continue,

    /// return expr;
    Return(Option<HirExpr>),

    /// fn name(params) -> ret { body }
    FnDef {
        name: String,
        params: Vec<(String, Option<String>)>, // (name, type)
        body: Vec<HirStatement>,
        return_type: Option<String>,
    },

    /// struct name { fields }
    StructDef {
        name: String,
        fields: Vec<(String, String)>, // (field_name, field_type)
    },

    /// trait name { methods }
    TraitDef {
        name: String,
        methods: Vec<HirTraitMethod>,
    },

    /// impl Trait for Type { methods }
    TraitImpl {
        trait_name: String,
        type_name: String,
        methods: Vec<HirImplMethod>,
    },
}

/// A method in a trait definition
#[derive(Debug, Clone)]
pub struct HirTraitMethod {
    pub name: String,
    pub params: Vec<(String, Option<String>)>, // (name, type)
    pub return_type: Option<String>,
    pub has_default: bool,
    pub body: Option<Vec<HirStatement>>,
}

/// A method in a trait implementation
#[derive(Debug, Clone)]
pub struct HirImplMethod {
    pub name: String,
    pub params: Vec<(String, Option<String>)>, // (name, type)
    pub return_type: Option<String>,
    pub body: Vec<HirStatement>,
}

/// A HIR expression
#[derive(Debug, Clone)]
pub enum HirExpr {
    /// String literal
    StringLit(String),

    /// Integer literal
    IntLit(i64),

    /// Boolean literal
    BoolLit(bool),

    /// Array literal vec![...]
    ArrayLit(Vec<HirExpr>),

    /// Index access array[index]
    Index {
        array: Box<HirExpr>,
        index: Box<HirExpr>,
    },

    /// Variable reference
    Var(String),

    /// Field access: expr.field
    Field {
        object: Box<HirExpr>,
        field: String,
    },

    /// Method call: expr.method(args)
    MethodCall {
        receiver: Box<HirExpr>,
        method: String,
        args: Vec<HirExpr>,
    },

    /// Function call: func(args)
    Call {
        func: String,
        args: Vec<HirExpr>,
    },

    /// Binary operation
    BinOp {
        op: BinOp,
        left: Box<HirExpr>,
        right: Box<HirExpr>,
    },

    /// Reference: &expr or &mut expr
    Ref {
        mutable: bool,
        expr: Box<HirExpr>,
    },

    /// Range for loops: start..end or start..=end
    Range {
        start: Box<HirExpr>,
        end: Box<HirExpr>,
        inclusive: bool,
    },

    /// Struct literal: TypeName { field: value, ... }
    StructLit {
        type_name: String,
        fields: Vec<String>,
        args: Vec<HirExpr>,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl From<crate::morphology::lexicon::BinaryOp> for BinOp {
    fn from(op: crate::morphology::lexicon::BinaryOp) -> Self {
        use crate::morphology::lexicon::BinaryOp as MorphOp;
        match op {
            MorphOp::Add => BinOp::Add,
            MorphOp::Sub => BinOp::Sub,
            MorphOp::Mul => BinOp::Mul,
            MorphOp::Div => BinOp::Div,
            MorphOp::Mod => BinOp::Mod,
            MorphOp::Eq => BinOp::Eq,
            MorphOp::Ne => BinOp::Ne,
            MorphOp::Lt => BinOp::Lt,
            MorphOp::Le => BinOp::Le,
            MorphOp::Gt => BinOp::Gt,
            MorphOp::Ge => BinOp::Ge,
            MorphOp::And => BinOp::And,
            MorphOp::Or => BinOp::Or,
        }
    }
}

/// Lower analyzed program to HIR
pub fn lower_to_hir(analyzed: &AnalyzedProgram) -> HirProgram {
    let mut statements = Vec::new();

    for stmt in &analyzed.statements {
        if let Some(hir_stmt) = lower_statement(stmt) {
            statements.push(hir_stmt);
        }
    }

    HirProgram { statements }
}

fn lower_statement(stmt: &AnalyzedStatement) -> Option<HirStatement> {
    match &stmt.kind {
        StatementKind::Binding { name, .. } => {
            // Get the value expression (second expression in the list)
            let value = if stmt.expressions.len() > 1 {
                lower_expr(&stmt.expressions[1])
            } else {
                HirExpr::IntLit(0) // Default
            };

            // Arrays need to be mutable for push/pop operations
            let is_array = matches!(value, HirExpr::ArrayLit(_));

            Some(HirStatement::Let {
                name: name.clone(),
                value,
                mutable: is_array,
            })
        }

        StatementKind::Print => {
            let args: Vec<HirExpr> = stmt.expressions.iter()
                .map(|e| lower_expr(e))
                .collect();

            Some(HirStatement::Print { args })
        }

        StatementKind::Expression => {
            if let Some(first) = stmt.expressions.first() {
                Some(HirStatement::Expr(lower_expr(first)))
            } else {
                None
            }
        }

        StatementKind::Query => {
            // For now, queries become print statements
            let args: Vec<HirExpr> = stmt.expressions.iter()
                .map(|e| lower_expr(e))
                .collect();

            Some(HirStatement::Print { args })
        }

        StatementKind::If { condition, then_body, else_body } => {
            Some(HirStatement::If {
                condition: lower_expr(condition),
                then_body: then_body.iter()
                    .filter_map(|s| lower_statement(s))
                    .collect(),
                else_body: else_body.as_ref().map(|stmts| {
                    stmts.iter()
                        .filter_map(|s| lower_statement(s))
                        .collect()
                }),
            })
        }

        StatementKind::While { condition, body } => {
            Some(HirStatement::While {
                condition: lower_expr(condition),
                body: body.iter()
                    .filter_map(|s| lower_statement(s))
                    .collect(),
            })
        }

        StatementKind::For { variable, iterator, body } => {
            Some(HirStatement::For {
                variable: variable.clone(),
                iterator: lower_expr(iterator),
                body: body.iter()
                    .filter_map(|s| lower_statement(s))
                    .collect(),
            })
        }

        StatementKind::Match { scrutinee, arms } => {
            Some(HirStatement::Match {
                scrutinee: lower_expr(scrutinee),
                arms: arms.iter()
                    .map(|(pattern, body)| {
                        let pattern_expr = lower_expr(pattern);
                        let body_stmts = body.iter()
                            .filter_map(|s| lower_statement(s))
                            .collect();
                        (pattern_expr, body_stmts)
                    })
                    .collect(),
            })
        }

        StatementKind::Break => Some(HirStatement::Break),

        StatementKind::Continue => Some(HirStatement::Continue),

        StatementKind::Return { value } => {
            Some(HirStatement::Return(value.as_ref().map(|v| lower_expr(v))))
        }

        StatementKind::FunctionDef { name, params, body, return_type } => {
            Some(HirStatement::FnDef {
                name: name.clone(),
                params: params.iter()
                    .map(|(n, t)| (n.clone(), t.as_ref().map(|ty| ty.to_rust().to_string())))
                    .collect(),
                body: body.iter()
                    .filter_map(|s| lower_statement(s))
                    .collect(),
                return_type: return_type.as_ref().map(|ty| ty.to_rust().to_string()),
            })
        }

        StatementKind::TypeDefinition { name, fields } => {
            Some(HirStatement::StructDef {
                name: name.clone(),
                fields: fields.iter()
                    .map(|(field_name, field_type)| (field_name.clone(), field_type.to_rust().to_string()))
                    .collect(),
            })
        }

        StatementKind::TraitDefinition { name, methods } => {
            Some(HirStatement::TraitDef {
                name: name.clone(),
                methods: methods.iter().map(|method| {
                    HirTraitMethod {
                        name: method.name.clone(),
                        params: method.params.iter()
                            .map(|(param_name, param_type)| (param_name.clone(), Some(param_type.to_rust().to_string())))
                            .collect(),
                        return_type: None, // We'll infer this later if needed
                        has_default: method.is_default,
                        body: method.body.as_ref().map(|body_stmts| {
                            body_stmts.iter().filter_map(lower_statement).collect()
                        }),
                    }
                }).collect(),
            })
        }

        StatementKind::TraitImplementation { trait_name, type_name, methods } => {
            Some(HirStatement::TraitImpl {
                trait_name: trait_name.clone(),
                type_name: type_name.clone(),
                methods: methods.iter().map(|method| {
                    HirImplMethod {
                        name: method.name.clone(),
                        params: method.params.iter()
                            .map(|(param_name, param_type)| (param_name.clone(), Some(param_type.to_rust().to_string())))
                            .collect(),
                        return_type: None, // We'll infer this later if needed
                        body: method.body.iter().filter_map(lower_statement).collect(),
                    }
                }).collect(),
            })
        }
    }
}

fn lower_expr(expr: &AnalyzedExpr) -> HirExpr {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => HirExpr::StringLit(s.clone()),
        AnalyzedExprKind::NumberLiteral(n) => HirExpr::IntLit(*n),
        AnalyzedExprKind::BooleanLiteral(b) => HirExpr::BoolLit(*b),
        AnalyzedExprKind::ArrayLiteral(elements) => {
            HirExpr::ArrayLit(elements.iter().map(lower_expr).collect())
        }
        AnalyzedExprKind::IndexAccess { array, index } => {
            HirExpr::Index {
                array: Box::new(lower_expr(array)),
                index: Box::new(lower_expr(index)),
            }
        }
        AnalyzedExprKind::MethodCall { receiver, method, args } => {
            HirExpr::MethodCall {
                receiver: Box::new(lower_expr(receiver)),
                method: method.clone(),
                args: args.iter().map(lower_expr).collect(),
            }
        }
        AnalyzedExprKind::TraitMethodCall { receiver, method_name, args, .. } => {
            // Trait method calls lower to regular method calls in Rust
            HirExpr::MethodCall {
                receiver: Box::new(lower_expr(receiver)),
                method: method_name.clone(),
                args: args.iter().map(lower_expr).collect(),
            }
        }
        AnalyzedExprKind::Variable(name) => HirExpr::Var(name.clone()),
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            HirExpr::Field {
                object: Box::new(lower_expr(owner)),
                field: property.clone(),
            }
        }
        AnalyzedExprKind::VerbCall { verb, args } => {
            HirExpr::Call {
                func: verb.clone(),
                args: args.iter().map(|a| lower_expr(a)).collect(),
            }
        }
        AnalyzedExprKind::FunctionCall { func, args } => {
            HirExpr::Call {
                func: func.clone(),
                args: args.iter().map(|a| lower_expr(a)).collect(),
            }
        }
        AnalyzedExprKind::BinOp { left, op, right } => {
            HirExpr::BinOp {
                op: (*op).into(),
                left: Box::new(lower_expr(left)),
                right: Box::new(lower_expr(right)),
            }
        }
        AnalyzedExprKind::UnaryOp { op, operand } => {
            // For now, treat unary ops as BinOp with a default value
            // TODO: Add proper UnaryOp to HirExpr
            match op {
                crate::morphology::lexicon::UnaryOp::Not => {
                    // !x is equivalent to x == false for booleans
                    HirExpr::BinOp {
                        op: BinOp::Eq,
                        left: Box::new(lower_expr(operand)),
                        right: Box::new(HirExpr::BoolLit(false)),
                    }
                }
                crate::morphology::lexicon::UnaryOp::Neg => {
                    // -x is equivalent to 0 - x
                    HirExpr::BinOp {
                        op: BinOp::Sub,
                        left: Box::new(HirExpr::IntLit(0)),
                        right: Box::new(lower_expr(operand)),
                    }
                }
            }
        }
        AnalyzedExprKind::Range { start, end, inclusive } => {
            HirExpr::Range {
                start: Box::new(lower_expr(start)),
                end: Box::new(lower_expr(end)),
                inclusive: *inclusive,
            }
        }
        AnalyzedExprKind::StructInstantiation { type_name, fields, args } => {
            HirExpr::StructLit {
                type_name: type_name.clone(),
                fields: fields.clone(),
                args: args.iter().map(lower_expr).collect(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::build_ast;
    use crate::semantic::analyze_program;

    #[test]
    fn test_lower_hello() {
        let ast = build_ast("«χαῖρε» λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let hir = lower_to_hir(&analyzed);

        assert_eq!(hir.statements.len(), 1);
        assert!(matches!(hir.statements[0], HirStatement::Print { .. }));
    }

    #[test]
    fn test_lower_binding() {
        let ast = build_ast("ξ πέντε ἔστω.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let hir = lower_to_hir(&analyzed);

        assert!(matches!(
            &hir.statements[0],
            HirStatement::Let { name, .. } if name == "ξ"
        ));
    }

    #[test]
    fn test_lower_number_literal() {
        let ast = build_ast("42 λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let hir = lower_to_hir(&analyzed);

        if let HirStatement::Print { args } = &hir.statements[0] {
            assert!(matches!(args[0], HirExpr::IntLit(42)));
        } else {
            panic!("Expected Print statement");
        }
    }
}
