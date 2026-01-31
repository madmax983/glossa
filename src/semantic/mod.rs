//! Semantic analysis for ΓΛΩΣΣΑ
//!
//! This module implements the semantic analysis pipeline, which transforms the raw AST
//! into a typed, resolved representation ready for intermediate code generation.
//!
//! # The Analysis Pipeline
//!
//! 1. **Morphological Analysis**: Raw words are analyzed for case, gender, number, etc.
//!    (handled by the `morphology` module).
//! 2. **Slot-Based Assembly**: The [`Assembler`] routes these words into grammatical slots
//!    (Subject, Object, Verb) based on their case endings. This provides the
//!    language's signature free word order.
//! 3. **Pattern Recognition**: The assembled sentence is classified into a statement kind
//!    (Binding, Print, If, etc.) based on the verb and constituents.
//! 4. **Name Resolution**: Variables are looked up in the [`Scope`] to ensure they exist.
//! 5. **Type Inference**: Types are inferred from usage and lexical definitions.
//!
//! # The Assembler Approach
//!
//! Unlike traditional parsers that rely on fixed word positions (e.g., "verb follows subject"),
//! ΓΛΩΣΣΑ uses the `Assembler` to assemble sentences based on grammatical *roles*.
//!
//! ```text
//! "ὁ ἄνθρωπος τὸν λόγον λέγει"
//!      ↓           ↓       ↓
//! [Nominative] [Accusative] [Verb]
//!      ↓           ↓       ↓
//!   Subject      Object   Action
//! ```
//!
//! This allows for authentic Greek syntax where emphasis is conveyed through word order
//! without changing the semantic meaning.

mod agreement;
pub mod assembler;
pub(crate) mod control_flow;
pub(crate) mod conversion;
pub(crate) mod declarations;
pub mod disambiguation;
pub(crate) mod expressions;
pub(crate) mod patterns;
mod resolver;
mod types;

pub use agreement::*;
pub use assembler::{
    AssembledStatement, Assembler, AssemblyError, Constituent, Literal, VerbConstituent,
};
pub use disambiguation::{DisambiguationContext, analyze_article, disambiguate, resolve_best};
pub use resolver::*;
pub use types::*;

use crate::ast::{Expr, Program, Statement};
use crate::errors::GlossaError;
use smol_str::SmolStr;

use self::control_flow::analyze_control_flow;
use self::conversion::{classify_value_expression, convert_assembled_to_analyzed};
use self::declarations::{analyze_trait_definition, analyze_trait_impl, analyze_type_definition};
use self::expressions::feed_expr_to_assembler_with_context;
use self::patterns::{try_parse_struct_instantiation, try_parse_trait_method_call};

/// Perform semantic analysis on a program using the slot-based assembler
/// This is the primary entry point that provides word-order independence
pub fn analyze_program(program: &Program) -> Result<AnalyzedProgram, GlossaError> {
    let mut scope = Scope::new();
    let mut analyzed_statements = Vec::new();

    for stmt in &program.statements {
        // Handle type definitions
        if let Statement::TypeDefinition(type_def) = stmt {
            analyzed_statements.push(analyze_type_definition(type_def, &mut scope)?);
            continue;
        }

        // Handle trait definitions
        if let Statement::TraitDefinition(trait_def) = stmt {
            analyzed_statements.push(analyze_trait_definition(trait_def, &mut scope)?);
            continue;
        }

        // Handle trait implementations
        if let Statement::TraitImpl(trait_impl) = stmt {
            analyzed_statements.push(analyze_trait_impl(trait_impl, &mut scope)?);
            continue;
        }

        // Check if this is a control flow construct
        if let Some(control_flow_stmt) = analyze_control_flow(stmt, &mut scope)? {
            // If it's a function definition, register it in the scope
            if let StatementKind::FunctionDef {
                name,
                params,
                return_type,
                ..
            } = &control_flow_stmt.kind
            {
                let param_types: Vec<GlossaType> = params
                    .iter()
                    .map(|(_, ty)| ty.clone().unwrap_or(GlossaType::Unknown))
                    .collect();
                scope.define_function(name.to_string(), param_types, return_type.clone());
            }
            analyzed_statements.push(control_flow_stmt);
        } else {
            // Check for struct instantiation pattern BEFORE assembler
            // Pattern: var_name νέον TypeName args... ἔστω
            if let Some(struct_inst) = try_parse_struct_instantiation(stmt, &mut scope)? {
                analyzed_statements.push(struct_inst);
            }
            // Check for trait method call pattern BEFORE assembler
            // Pattern: method_name receiver
            else if let Some(method_call) = try_parse_trait_method_call(stmt, &mut scope)? {
                analyzed_statements.push(method_call);
            } else {
                // Use the assembler-based approach for regular statements
                let assembled = analyze_single_statement_with_assembler(stmt)?;
                let analyzed = convert_assembled_to_analyzed(&assembled, &mut scope)?;
                analyzed_statements.push(analyzed);
            }
        }
    }

    Ok(AnalyzedProgram {
        statements: analyzed_statements,
        scope,
    })
}

/// Legacy analysis method that doesn't use the assembler
/// Kept for comparison and fallback purposes
pub fn analyze_program_legacy(program: &Program) -> Result<AnalyzedProgram, GlossaError> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(program)
}

/// Analyze a single statement using the slot-based assembler
fn analyze_single_statement_with_assembler(
    stmt: &Statement,
) -> Result<AssembledStatement, GlossaError> {
    let mut asm = Assembler::new();
    asm.set_query(stmt.is_query());
    asm.set_propagate(stmt.is_propagate());

    // Disambiguation context accumulator - articles set context for following words
    let mut current_context = DisambiguationContext::new();

    // Feed each expression/term to the assembler with disambiguation
    // Process all clauses - they're separated by commas in the grammar
    for clause in stmt.clauses() {
        for expr in &clause.expressions {
            feed_expr_to_assembler_with_context(&mut asm, expr, &mut current_context)?;
        }
    }

    // Finalize the statement
    match asm.finalize() {
        Ok(assembled) => Ok(assembled),
        Err(e) => Err(GlossaError::semantic(e.to_string())),
    }
}

/// Analyzed program with resolved names and types
#[derive(Debug, Clone)]
pub struct AnalyzedProgram {
    pub statements: Vec<AnalyzedStatement>,
    pub scope: Scope,
}

/// Analyzed statement
#[derive(Debug, Clone)]
pub struct AnalyzedStatement {
    pub kind: StatementKind,
    pub expressions: Vec<AnalyzedExpr>,
}

/// The kind of statement
#[derive(Debug, Clone)]
pub enum StatementKind {
    /// Variable binding: ξ πέντε ἔστω
    Binding {
        name: SmolStr,
        value_type: GlossaType,
        mutable: bool,
    },
    /// Assignment: ξ δέκα γίγνεται
    Assignment {
        name: SmolStr,
        value_type: GlossaType,
    },
    /// Print statement: «χαῖρε» λέγε
    Print,
    /// Expression statement
    Expression,
    /// Query: ξ?
    Query,
    /// If conditional: εἰ condition, body [εἰ δὲ μή, else_body]
    If {
        condition: Box<AnalyzedExpr>,
        then_body: Vec<AnalyzedStatement>,
        else_body: Option<Vec<AnalyzedStatement>>,
    },
    /// While loop: ἕως condition, body
    While {
        condition: Box<AnalyzedExpr>,
        body: Vec<AnalyzedStatement>,
    },
    /// For loop: διά/ἀπό...μέχρι
    For {
        variable: SmolStr,
        iterator: Box<AnalyzedExpr>,
        body: Vec<AnalyzedStatement>,
    },
    /// Match expression: κατά scrutinee { arms }
    Match {
        scrutinee: Box<AnalyzedExpr>,
        arms: Vec<(AnalyzedExpr, Vec<AnalyzedStatement>)>,
    },
    /// Break: παῦε
    Break,
    /// Continue: συνέχιζε
    Continue,
    /// Return: δός value
    Return { value: Option<Box<AnalyzedExpr>> },
    /// Function definition: name ὁρίζειν params· body
    FunctionDef {
        name: SmolStr,
        params: Vec<(SmolStr, Option<GlossaType>)>,
        body: Vec<AnalyzedStatement>,
        return_type: Option<GlossaType>,
    },
    /// Type definition: εἶδος name ὁρίζειν { fields }
    TypeDefinition {
        name: SmolStr,
        fields: Vec<(SmolStr, GlossaType)>,
    },
    /// Trait definition: χαρακτήρ name ὁρίζειν { methods }
    TraitDefinition {
        name: SmolStr,
        methods: Vec<AnalyzedTraitMethod>,
    },
    /// Trait implementation: εἶδος Type τῷ Trait ἐμπίπτειν { methods }
    TraitImplementation {
        trait_name: SmolStr,
        type_name: SmolStr,
        methods: Vec<AnalyzedImplMethod>,
    },
}

/// An analyzed method in a trait definition
#[derive(Debug, Clone)]
pub struct AnalyzedTraitMethod {
    pub name: SmolStr,
    pub params: Vec<(SmolStr, GlossaType)>,
    pub is_default: bool,
    pub body: Option<Vec<AnalyzedStatement>>, // Some for default methods, None for required
    pub return_type: Option<GlossaType>,
}

/// An analyzed method in a trait implementation
#[derive(Debug, Clone)]
pub struct AnalyzedImplMethod {
    pub name: SmolStr,
    pub params: Vec<(SmolStr, GlossaType)>,
    pub body: Vec<AnalyzedStatement>,
    pub return_type: Option<GlossaType>,
}

/// Iterator operation for AnalyzedExpr
#[derive(Debug, Clone)]
pub enum AnalyzedIteratorOp {
    /// .iter() - create iterator
    Iter,
    /// .map(closure) - transform elements
    Map(Box<AnalyzedExpr>),
    /// .filter(closure) - select elements
    Filter(Box<AnalyzedExpr>),
    /// .find(closure) - find first matching element
    Find(Box<AnalyzedExpr>),
    /// .fold(init, closure) - reduce to single value
    Fold {
        init: Box<AnalyzedExpr>,
        closure: Box<AnalyzedExpr>,
    },
    /// .any(closure) - test if any element matches
    Any(Box<AnalyzedExpr>),
    /// .all(closure) - test if all elements match
    All(Box<AnalyzedExpr>),
    /// .collect() - collect into collection
    Collect,
}

/// Analyzed expression with type information
#[derive(Debug, Clone)]
pub struct AnalyzedExpr {
    pub expr: AnalyzedExprKind,
    pub glossa_type: GlossaType,
}

/// Kind of analyzed expression
#[derive(Debug, Clone)]
pub enum AnalyzedExprKind {
    StringLiteral(String),
    NumberLiteral(i64),
    BooleanLiteral(bool),
    Variable(SmolStr),
    PropertyAccess {
        owner: Box<AnalyzedExpr>,
        property: SmolStr,
    },
    VerbCall {
        verb: SmolStr,
        args: Vec<AnalyzedExpr>,
    },
    /// Binary operation (arithmetic, comparison, boolean)
    BinOp {
        left: Box<AnalyzedExpr>,
        op: crate::morphology::lexicon::BinaryOp,
        right: Box<AnalyzedExpr>,
    },
    /// Unary operation (negation)
    UnaryOp {
        op: crate::morphology::lexicon::UnaryOp,
        operand: Box<AnalyzedExpr>,
    },
    /// Range expression for loops (start..end or start..=end)
    Range {
        start: Box<AnalyzedExpr>,
        end: Box<AnalyzedExpr>,
        inclusive: bool,
    },
    /// Array literal [1, 2, 3]
    ArrayLiteral(Vec<AnalyzedExpr>),
    /// Some(value) - `Option<T>` constructor
    Some(Box<AnalyzedExpr>),
    /// None - `Option<T>` empty value
    None,
    /// Ok(value) - `Result<T,E>` success constructor
    Ok(Box<AnalyzedExpr>),
    /// Err(error) - `Result<T,E>` error constructor
    Err(Box<AnalyzedExpr>),
    /// Unwrap operator (!) - confident extraction from `Option`/`Result`
    Unwrap(Box<AnalyzedExpr>),
    /// Try operator (`;` after `Option`/`Result`) - propagates None/Err upward
    Try(Box<AnalyzedExpr>),
    /// Index access `array[index]`
    IndexAccess {
        array: Box<AnalyzedExpr>,
        index: Box<AnalyzedExpr>,
    },
    /// Function call to user-defined function
    FunctionCall {
        func: SmolStr,
        args: Vec<AnalyzedExpr>,
    },
    /// Method call receiver.method(args)
    MethodCall {
        receiver: Box<AnalyzedExpr>,
        method: SmolStr,
        args: Vec<AnalyzedExpr>,
    },
    /// Trait method call receiverου method args (from trait impl)
    TraitMethodCall {
        receiver: Box<AnalyzedExpr>,
        trait_name: SmolStr,
        method_name: SmolStr,
        args: Vec<AnalyzedExpr>,
    },
    /// Struct instantiation Type { field: value, ... }
    StructInstantiation {
        type_name: SmolStr,
        fields: Vec<SmolStr>, // Field names from struct definition
        args: Vec<AnalyzedExpr>,
    },
    /// Lambda/closure |params| body
    Lambda {
        params: Vec<SmolStr>,
        body: Box<AnalyzedExpr>,
        capture_mode: crate::ast::CaptureMode,
    },
    /// Iterator chain collection.iter().map(...).filter(...)
    IteratorChain {
        collection: Box<AnalyzedExpr>,
        ops: Vec<AnalyzedIteratorOp>,
    },
    /// Literal value (used in iterator ops, different from specific literals above)
    Literal(i64),
    /// Collection constructor (HashSet::new(), HashMap::new())
    CollectionNew {
        collection_type: String,
    },
    /// Collection contains check (set.contains(&x), map.contains_key(&k))
    CollectionContains {
        collection: Box<AnalyzedExpr>,
        element: Box<AnalyzedExpr>,
        is_map: bool,
    },
}

/// Semantic analyzer state
pub struct SemanticAnalyzer {
    scope: Scope,
    errors: Vec<GlossaError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            scope: Scope::new(),
            errors: Vec::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<AnalyzedProgram, GlossaError> {
        let mut analyzed_statements = Vec::new();

        for stmt in &program.statements {
            match self.analyze_statement(stmt) {
                Ok(analyzed) => analyzed_statements.push(analyzed),
                Err(e) => self.errors.push(e),
            }
        }

        if !self.errors.is_empty() {
            return Err(self.errors[0].clone());
        }

        Ok(AnalyzedProgram {
            statements: analyzed_statements,
            scope: self.scope.clone(),
        })
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<AnalyzedStatement, GlossaError> {
        // Determine statement kind by analyzing expressions
        // Collect expressions from all clauses
        let exprs: Vec<&Expr> = stmt.expressions().collect();

        if exprs.is_empty() {
            return Ok(AnalyzedStatement {
                kind: StatementKind::Expression,
                expressions: vec![],
            });
        }

        // Analyze the first (and usually only) expression
        let first_expr = exprs[0];
        let (kind, analyzed_exprs) = self.analyze_expression_list(first_expr, stmt.is_query())?;

        Ok(AnalyzedStatement {
            kind,
            expressions: analyzed_exprs,
        })
    }

    fn analyze_expression_list(
        &mut self,
        expr: &Expr,
        is_query: bool,
    ) -> Result<(StatementKind, Vec<AnalyzedExpr>), GlossaError> {
        match expr {
            Expr::Phrase(terms) => self.analyze_phrase(terms, is_query),
            _ => {
                let analyzed = self.analyze_single_expr(expr)?;
                let kind = if is_query {
                    StatementKind::Query
                } else {
                    StatementKind::Expression
                };
                Ok((kind, vec![analyzed]))
            }
        }
    }

    fn analyze_phrase(
        &mut self,
        terms: &[Expr],
        is_query: bool,
    ) -> Result<(StatementKind, Vec<AnalyzedExpr>), GlossaError> {
        // Look for patterns: binding, print, property access

        // Check for struct instantiation pattern: var_name νέον TypeName args... ἔστω
        // Must be checked before assembler since TypeName might be a Latin identifier
        if terms.len() >= 4 {
            // Check if last term is ἔστω (binding verb)
            if let Expr::Word(last_word) = terms.last().unwrap()
                && crate::morphology::lexicon::is_binding_verb(&last_word.normalized)
            {
                // Check if second term is νέον (new)
                if let Expr::Word(second_word) = &terms[1] {
                    let normalized_adj = &second_word.normalized;
                    if normalized_adj == "νεος" || normalized_adj == "νεον" {
                        // This looks like: var_name νέον TypeName args... ἔστω
                        if let Expr::Word(var_word) = &terms[0]
                            && let Expr::Word(type_word) = &terms[2]
                        {
                            let var_name = &var_word.normalized;
                            let type_name = &type_word.normalized;

                            // Check if the type exists
                            if let Some(struct_type) = self.scope.lookup_type(type_name).cloned() {
                                // Extract field names from struct type
                                let field_names: Vec<SmolStr> =
                                    if let GlossaType::Struct { fields, .. } = &struct_type {
                                        fields.iter().map(|(name, _)| name.clone()).collect()
                                    } else {
                                        vec![]
                                    };

                                // Collect constructor arguments (everything between type_name and ἔστω)
                                let mut args = Vec::new();
                                for term in &terms[3..terms.len() - 1] {
                                    args.push(self.analyze_single_expr(term)?);
                                }

                                // Build struct instantiation
                                let struct_inst = AnalyzedExpr {
                                    expr: AnalyzedExprKind::StructInstantiation {
                                        type_name: type_name.clone(),
                                        fields: field_names,
                                        args,
                                    },
                                    glossa_type: struct_type.clone(),
                                };

                                // Register variable in scope with correct type
                                self.scope.define(var_name.to_string(), struct_type.clone());

                                return Ok((
                                    StatementKind::Binding {
                                        name: var_name.clone(),
                                        value_type: struct_type.clone(),
                                        mutable: false,
                                    },
                                    vec![
                                        AnalyzedExpr {
                                            expr: AnalyzedExprKind::Variable(var_name.clone()),
                                            glossa_type: struct_type.clone(),
                                        },
                                        struct_inst,
                                    ],
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Check for trait method call pattern: method_name receiver (e.g., "show π")
        // This must be checked before verb analysis since method names may not be Greek verbs
        if terms.len() == 2
            && let (Expr::Word(method_word), Expr::Word(receiver_word)) = (&terms[0], &terms[1])
        {
            let method_name = &method_word.normalized;
            let receiver_name = &receiver_word.normalized;

            // Check if receiver is a variable in scope
            if let Some(receiver_type) = self.scope.lookup(receiver_name)
                && let GlossaType::Struct {
                    name: type_name, ..
                } = receiver_type
            {
                // Check if this type has a trait method with this name
                if self.scope.has_trait_method(type_name, method_name) {
                    let receiver = AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(receiver_name.clone()),
                        glossa_type: receiver_type.clone(),
                    };

                    let method_call = AnalyzedExpr {
                        expr: AnalyzedExprKind::MethodCall {
                            receiver: Box::new(receiver),
                            method: method_name.clone(),
                            args: vec![],
                        },
                        glossa_type: GlossaType::Unit,
                    };

                    return Ok((StatementKind::Expression, vec![method_call]));
                }
            }
        }

        // Find verb position and type
        let mut verb_idx = None;
        let mut is_binding = false;
        let mut is_print = false;

        for (i, term) in terms.iter().enumerate() {
            if let Expr::Word(w) = term {
                let normalized = &w.normalized;
                if crate::morphology::lexicon::is_binding_verb(normalized) {
                    verb_idx = Some(i);
                    is_binding = true;
                    break;
                }
                if crate::morphology::lexicon::is_print_verb(normalized) {
                    verb_idx = Some(i);
                    is_print = true;
                    break;
                }
            }
        }

        if is_binding {
            // Pattern: [μετά] name value... ἔστω
            // e.g., ξ πέντε ἔστω
            // e.g., μετά ξ πέντε ἔστω (mutable)
            // e.g., τοπικον ξ ἓν ἄθροισμα ἔστω
            if terms.len() >= 3 {
                // Check for μετά mutable marker
                let (is_mutable, name_start) = if let Expr::Word(w) = &terms[0] {
                    if crate::morphology::lexicon::is_mutable_marker(&w.normalized) {
                        (true, 1)
                    } else {
                        (false, 0)
                    }
                } else {
                    (false, 0)
                };

                let name = self.extract_name(&terms[name_start])?;

                // Collect all terms between the name and the ἔστω verb
                let verb_position = verb_idx.unwrap();
                let value_start = name_start + 1;
                let value_terms = &terms[value_start..verb_position];

                // Analyze the value expression using the assembler
                // Create a phrase expression from the value terms and analyze it
                let value_expr = if value_terms.len() == 1 {
                    // Simple case: single term value
                    self.analyze_single_expr(&value_terms[0])?
                } else {
                    // Complex case: multiple terms (e.g., "ξ ἓν ἄθροισμα")
                    // Use assembler to properly analyze the expression
                    let phrase_expr = Expr::Phrase(value_terms.to_vec());

                    // Create a synthetic statement to analyze through assembler
                    use crate::ast::{Clause, Statement};
                    let synthetic_stmt = Statement::Regular {
                        clauses: vec![Clause {
                            expressions: vec![phrase_expr],
                        }],
                        is_query: false,
                        is_propagate: false,
                    };

                    // Analyze through assembler
                    let assembled = analyze_single_statement_with_assembler(&synthetic_stmt)?;

                    // Convert to AnalyzedExpr using pattern detection
                    classify_value_expression(&assembled, &mut self.scope)?
                };

                // Register in scope (mutable if μετά marker present)
                if is_mutable {
                    self.scope
                        .define_mut(name.clone(), value_expr.glossa_type.clone());
                } else {
                    self.scope
                        .define(name.clone(), value_expr.glossa_type.clone());
                }

                return Ok((
                    StatementKind::Binding {
                        name: name.clone().into(),
                        value_type: value_expr.glossa_type.clone(),
                        mutable: is_mutable,
                    },
                    vec![
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(name.into()),
                            glossa_type: value_expr.glossa_type.clone(),
                        },
                        value_expr,
                    ],
                ));
            }
        }

        if is_print {
            // Pattern: value λέγε
            let mut args = Vec::new();
            for term in terms.iter().take(verb_idx.unwrap_or(terms.len())) {
                args.push(self.analyze_single_expr(term)?);
            }

            return Ok((StatementKind::Print, args));
        }

        // Default: analyze all terms
        let mut analyzed = Vec::new();
        for term in terms {
            analyzed.push(self.analyze_single_expr(term)?);
        }

        let kind = if is_query {
            StatementKind::Query
        } else {
            StatementKind::Expression
        };

        Ok((kind, analyzed))
    }

    fn analyze_single_expr(&self, expr: &Expr) -> Result<AnalyzedExpr, GlossaError> {
        match expr {
            Expr::StringLiteral(s) => Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::StringLiteral(s.clone()),
                glossa_type: GlossaType::String,
            }),

            Expr::NumberLiteral(n) => Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(*n),
                glossa_type: GlossaType::Number,
            }),

            Expr::BooleanLiteral(b) => Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(*b),
                glossa_type: GlossaType::Boolean,
            }),

            Expr::Word(w) => {
                // Check if it's a numeral word
                if let Some(value) = crate::morphology::lexicon::numeral_value(&w.normalized) {
                    return Ok(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(value),
                        glossa_type: GlossaType::Number,
                    });
                }

                // Check if it's a known variable
                if let Some(var_type) = self.scope.lookup(&w.normalized) {
                    return Ok(AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(w.normalized.clone()),
                        glossa_type: var_type.clone(),
                    });
                }

                // Unknown word - treat as variable reference
                Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(w.normalized.clone()),
                    glossa_type: GlossaType::Unknown,
                })
            }

            Expr::Phrase(terms) => {
                // Nested phrase - analyze recursively
                if terms.len() == 1 {
                    return self.analyze_single_expr(&terms[0]);
                }

                // For now, return first term's type
                let first = self.analyze_single_expr(&terms[0])?;
                Ok(first)
            }

            _ => Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("_".into()),
                glossa_type: GlossaType::Unknown,
            }),
        }
    }

    fn extract_name(&self, expr: &Expr) -> Result<String, GlossaError> {
        match expr {
            Expr::Word(w) => Ok(w.normalized.to_string()),
            _ => Err(GlossaError::semantic("Expected a word for variable name")),
        }
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::build_ast;

    #[test]
    fn test_analyze_hello() {
        let ast = build_ast("«χαῖρε» λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        assert_eq!(analyzed.statements.len(), 1);
        assert!(matches!(analyzed.statements[0].kind, StatementKind::Print));
    }

    #[test]
    fn test_analyze_binding() {
        let ast = build_ast("ξ πέντε ἔστω.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        assert!(matches!(
            &analyzed.statements[0].kind,
            StatementKind::Binding { name, .. } if name == "ξ"
        ));

        // Check that ξ is now in scope
        assert!(analyzed.scope.lookup("ξ").is_some());
    }

    #[test]
    fn test_analyze_variable_use() {
        let ast = build_ast("ξ πέντε ἔστω. ξ λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        assert_eq!(analyzed.statements.len(), 2);
        // Second statement should reference ξ with known type
        assert!(matches!(analyzed.statements[1].kind, StatementKind::Print));
    }

    #[test]
    fn test_analyze_string_literal() {
        let ast = build_ast("«χαῖρε κόσμε» λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        let first_expr = &analyzed.statements[0].expressions[0];
        assert_eq!(first_expr.glossa_type, GlossaType::String);
    }

    #[test]
    fn test_analyze_number_literal() {
        let ast = build_ast("42 λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        let first_expr = &analyzed.statements[0].expressions[0];
        assert_eq!(first_expr.glossa_type, GlossaType::Number);
    }
}
