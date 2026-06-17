//! The Narrator Tool ("Bard")
//!
//! This module implements the "Bard" functionality, which translates the semantic meaning
//! of a ΓΛΩΣΣΑ program into a readable English narrative, known as "The Scroll of Logic".
//!
//! # Purpose
//!
//! This tool serves two main purposes:
//! 1. **Debugging**: It allows developers to verify how the compiler is interpreting their code.
//!    If the English narrative doesn't match their intent, there's likely a parsing or semantic error.
//! 2. **Education**: It helps users understand the mapping between Ancient Greek syntax and
//!    computational logic.
//!
//! # How it works
//!
//! The `tell_tale` function takes an [`AnalyzedProgram`] (the output of the semantic analysis phase)
//! and recursively traverses the AST, generating a structured table (using `comfy_table`)
//! that classifies each statement by its "Act" (Type), "Script" (Description), and "Notes" (Properties).

use crate::semantic::CaptureMode;
use crate::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType,
};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use std::fmt::Write;

/// Tells the tale of the program in English, rendering the "Scroll of Logic".
///
/// This function exists to provide a human-readable translation of the internal
/// semantic program. By converting abstract syntax into a structured narrative table,
/// it serves as a powerful debugging tool: if the English story doesn't match the
/// developer's intent, they immediately know they have a logic or grammar error.
///
/// # Examples
///
/// ```
/// use glossa::parser::parse;
/// use glossa::semantic::analyze_program;
/// use glossa::tools::narrator::tell_tale;
///
/// let source = "ξ 5 ἔστω.";
/// let ast = parse(source).unwrap();
/// let analyzed = analyze_program(&ast).unwrap();
///
/// let narrative = tell_tale(&analyzed);
/// assert!(narrative.contains("BIND"));
/// assert!(narrative.contains("Let `ξ` be 5"));
/// ```
pub fn tell_tale(program: &AnalyzedProgram) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Act")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("The Scroll of Logic")
                .add_attribute(Attribute::Bold)
                .fg(Color::Yellow),
            Cell::new("Notes")
                .add_attribute(Attribute::Bold)
                .fg(Color::Magenta),
        ]);

    for stmt in &program.statements {
        add_statement(&mut table, stmt, 0);
    }

    // Add a footer
    table.add_row(vec![
        Cell::new("FIN").fg(Color::DarkGrey),
        Cell::new("...and thus the ritual is complete.")
            .fg(Color::DarkGrey)
            .add_attribute(Attribute::Italic),
        Cell::new("").fg(Color::DarkGrey),
    ]);

    table.to_string()
}

/// Calculates the visual indentation string for nested statements.
///
/// This exists because the Scroll of Logic represents hierarchical control flow
/// (like `if` statements and loops) using horizontal spacing rather than
/// traditional block delimiters like braces.
///
/// # Examples
///
/// ```text
/// let level_zero = indent(0); // ""
/// let level_two = indent(2);  // "    "
/// ```
fn indent(level: usize) -> String {
    "  ".repeat(level)
}

/// Appends a semantic statement as a descriptive row in the narrative table.
///
/// Instead of merely printing the abstract syntax tree, this function translates
/// rigid compiler constructs into human-readable English "Acts". This bridges the gap
/// between the raw parsed data and the developer's original intent, helping users verify
/// what the compiler actually understood.
///
/// # Panics
///
/// This function does not panic, but relies on the underlying table implementation
/// to handle memory allocation for new rows.
fn add_statement(table: &mut Table, stmt: &AnalyzedStatement, level: usize) {
    let prefix = indent(level);

    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => add_binding(table, &prefix, name, value, *mutable),
        AnalyzedStatement::Assignment { name, value } => {
            add_assignment(table, &prefix, name, value)
        }
        AnalyzedStatement::Print(exprs) => add_print(table, &prefix, exprs),
        AnalyzedStatement::Expression(exprs) => add_expression(table, &prefix, exprs),
        AnalyzedStatement::Query(exprs) => add_query(table, &prefix, exprs),
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => add_if(table, &prefix, level, condition, then_body, else_body),
        AnalyzedStatement::While { condition, body } => {
            add_while(table, &prefix, level, condition, body)
        }
        AnalyzedStatement::For {
            variable,
            iterator,
            body,
        } => add_for(table, &prefix, level, variable, iterator, body),
        AnalyzedStatement::Match { scrutinee, arms } => {
            add_match(table, &prefix, level, scrutinee, arms)
        }
        AnalyzedStatement::Break => add_break(table, &prefix),
        AnalyzedStatement::Continue => add_continue(table, &prefix),
        AnalyzedStatement::Return { value } => add_return(table, &prefix, value),
        AnalyzedStatement::FunctionDef {
            name,
            params,
            body,
            return_type,
        } => add_function_def(table, &prefix, level, name, params, body, return_type),
        AnalyzedStatement::TypeDefinition { name, fields } => {
            add_type_def(table, &prefix, name, fields)
        }
        AnalyzedStatement::TraitDefinition { name, methods: _ } => {
            add_trait_def(table, &prefix, name)
        }
        AnalyzedStatement::TraitImplementation {
            trait_name,
            type_name,
            methods: _,
        } => add_trait_impl(table, &prefix, trait_name, type_name),
        AnalyzedStatement::TestDeclaration { name, body } => {
            add_test_decl(table, &prefix, level, name, body)
        }
    }
}

fn add_binding(table: &mut Table, prefix: &str, name: &str, value: &AnalyzedExpr, mutable: bool) {
    let mut val_str = String::with_capacity(32);
    tell_expr_into(value, &mut val_str);
    let script = format!("Let `{}` be {}.", name, val_str);
    let notes = if mutable { "Mutable" } else { "Immutable" };
    table.add_row(vec![
        Cell::new("BIND").fg(Color::Blue),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new(notes).fg(if mutable { Color::Red } else { Color::Green }),
    ]);
}

fn add_assignment(table: &mut Table, prefix: &str, name: &str, value: &AnalyzedExpr) {
    let mut val_str = String::with_capacity(32);
    tell_expr_into(value, &mut val_str);
    let script = format!("Update `{}` to {}.", name, val_str);
    table.add_row(vec![
        Cell::new("SET").fg(Color::Yellow),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Mutation").fg(Color::Red),
    ]);
}

fn format_exprs(exprs: &[AnalyzedExpr], buf: &mut String) {
    for (i, expr) in exprs.iter().enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        tell_expr_into(expr, buf);
    }
}

fn format_types(types: &[GlossaType], buf: &mut String) {
    for (i, ty) in types.iter().enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        tell_type_into(ty, buf);
    }
}

fn add_print(table: &mut Table, prefix: &str, exprs: &[AnalyzedExpr]) {
    let mut expr_str = String::with_capacity(exprs.len() * 16);
    format_exprs(exprs, &mut expr_str);
    let script = format!("Proclaim: {}", expr_str);
    table.add_row(vec![
        Cell::new("PRINT").fg(Color::Green),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("I/O").fg(Color::Cyan),
    ]);
}

fn add_expression(table: &mut Table, prefix: &str, exprs: &[AnalyzedExpr]) {
    let mut expr_str = String::with_capacity(exprs.len() * 16);
    format_exprs(exprs, &mut expr_str);
    let script = format!("Do: {}", expr_str);
    table.add_row(vec![
        Cell::new("EXPR").fg(Color::DarkGrey),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Side Effect").fg(Color::DarkGrey),
    ]);
}

fn add_query(table: &mut Table, prefix: &str, exprs: &[AnalyzedExpr]) {
    let mut expr_str = String::with_capacity(exprs.len() * 16);
    format_exprs(exprs, &mut expr_str);
    let script = format!("Query oracle: {}", expr_str);
    table.add_row(vec![
        Cell::new("QUERY").fg(Color::Magenta),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Debug").fg(Color::Yellow),
    ]);
}

fn add_if(
    table: &mut Table,
    prefix: &str,
    level: usize,
    condition: &AnalyzedExpr,
    then_body: &[AnalyzedStatement],
    else_body: &Option<Vec<AnalyzedStatement>>,
) {
    let mut expr_str = String::with_capacity(32);
    tell_expr_into(condition, &mut expr_str);
    let script = format!("If {} is true, then:", expr_str);
    table.add_row(vec![
        Cell::new("IF").fg(Color::Magenta),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Branch").fg(Color::Magenta),
    ]);

    for stmt in then_body {
        add_statement(table, stmt, level + 1);
    }

    if let Some(else_stmts) = else_body {
        table.add_row(vec![
            Cell::new("ELSE").fg(Color::Magenta),
            Cell::new(format!("{}Otherwise:", prefix)),
            Cell::new("Branch").fg(Color::Magenta),
        ]);
        for stmt in else_stmts {
            add_statement(table, stmt, level + 1);
        }
    }
}

fn add_while(
    table: &mut Table,
    prefix: &str,
    level: usize,
    condition: &AnalyzedExpr,
    body: &[AnalyzedStatement],
) {
    let mut expr_str = String::with_capacity(32);
    tell_expr_into(condition, &mut expr_str);
    let script = format!("While {} holds true:", expr_str);
    table.add_row(vec![
        Cell::new("WHILE").fg(Color::Magenta),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Loop").fg(Color::Magenta),
    ]);
    for stmt in body {
        add_statement(table, stmt, level + 1);
    }
}

fn add_for(
    table: &mut Table,
    prefix: &str,
    level: usize,
    variable: &str,
    iterator: &AnalyzedExpr,
    body: &[AnalyzedStatement],
) {
    let mut expr_str = String::with_capacity(32);
    tell_expr_into(iterator, &mut expr_str);
    let script = format!("For each `{}` in {}:", variable, expr_str);
    table.add_row(vec![
        Cell::new("FOR").fg(Color::Magenta),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Iteration").fg(Color::Magenta),
    ]);
    for stmt in body {
        add_statement(table, stmt, level + 1);
    }
}

fn add_match(
    table: &mut Table,
    prefix: &str,
    level: usize,
    scrutinee: &AnalyzedExpr,
    arms: &[(AnalyzedExpr, Vec<AnalyzedStatement>)],
) {
    let mut expr_str = String::with_capacity(32);
    tell_expr_into(scrutinee, &mut expr_str);
    let script = format!("Match on {}:", expr_str);
    table.add_row(vec![
        Cell::new("MATCH").fg(Color::Magenta),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Pattern").fg(Color::Magenta),
    ]);
    for (pat, body) in arms {
        let mut pat_str = String::with_capacity(32);
        tell_expr_into(pat, &mut pat_str);
        let case_script = format!("Case {}:", pat_str);
        table.add_row(vec![
            Cell::new("CASE").fg(Color::DarkMagenta),
            Cell::new(format!("{}{}", indent(level + 1), case_script)),
            Cell::new("Arm").fg(Color::DarkMagenta),
        ]);
        for stmt in body {
            add_statement(table, stmt, level + 2);
        }
    }
}

fn add_break(table: &mut Table, prefix: &str) {
    table.add_row(vec![
        Cell::new("BREAK").fg(Color::Red),
        Cell::new(format!("{}Break loop.", prefix)),
        Cell::new("Control").fg(Color::Red),
    ]);
}

fn add_continue(table: &mut Table, prefix: &str) {
    table.add_row(vec![
        Cell::new("CONT").fg(Color::Green),
        Cell::new(format!("{}Continue loop.", prefix)),
        Cell::new("Control").fg(Color::Green),
    ]);
}

fn add_return(table: &mut Table, prefix: &str, value: &Option<Box<AnalyzedExpr>>) {
    let script = if let Some(v) = value {
        let mut v_str = String::with_capacity(32);
        tell_expr_into(v, &mut v_str);
        format!("Return {}.", v_str)
    } else {
        "Return nothing.".to_string()
    };
    table.add_row(vec![
        Cell::new("RETURN").fg(Color::Yellow),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Exit").fg(Color::Yellow),
    ]);
}

fn add_function_def(
    table: &mut Table,
    prefix: &str,
    level: usize,
    name: &str,
    params: &[(smol_str::SmolStr, Option<GlossaType>)],
    body: &[AnalyzedStatement],
    return_type: &Option<GlossaType>,
) {
    let mut params_buf = String::with_capacity(params.len() * 16);
    for (i, (n, t)) in params.iter().enumerate() {
        if i > 0 {
            params_buf.push_str(", ");
        }
        let type_str = t.as_ref().map(tell_type).unwrap_or("unknown".to_string());
        let _ = write!(&mut params_buf, "{}: {}", n, type_str);
    }

    let ret_str = return_type
        .as_ref()
        .map(tell_type)
        .unwrap_or("Nothing".to_string());

    let script = format!("Define `{}` ({}) -> {}:", name, params_buf, ret_str);
    table.add_row(vec![
        Cell::new("FUNC").fg(Color::Cyan),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Definition").fg(Color::Cyan),
    ]);
    for stmt in body {
        add_statement(table, stmt, level + 1);
    }
}

fn add_type_def(
    table: &mut Table,
    prefix: &str,
    name: &str,
    fields: &[(smol_str::SmolStr, GlossaType)],
) {
    let mut fields_buf = String::with_capacity(fields.len() * 16);
    for (i, (n, t)) in fields.iter().enumerate() {
        if i > 0 {
            fields_buf.push_str(", ");
        }
        let _ = write!(&mut fields_buf, "{}: {}", n, tell_type(t));
    }
    let script = format!("Struct `{}` {{ {} }}", name, fields_buf);
    table.add_row(vec![
        Cell::new("TYPE").fg(Color::Blue),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Struct").fg(Color::Blue),
    ]);
}

fn add_trait_def(table: &mut Table, prefix: &str, name: &str) {
    let script = format!("Trait `{}`", name);
    table.add_row(vec![
        Cell::new("TRAIT").fg(Color::Blue),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Interface").fg(Color::Blue),
    ]);
}

fn add_trait_impl(table: &mut Table, prefix: &str, trait_name: &str, type_name: &str) {
    let script = format!("Impl `{}` for `{}`", trait_name, type_name);
    table.add_row(vec![
        Cell::new("IMPL").fg(Color::Blue),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Implementation").fg(Color::Blue),
    ]);
}

fn add_test_decl(
    table: &mut Table,
    prefix: &str,
    level: usize,
    name: &str,
    body: &[AnalyzedStatement],
) {
    let script = format!("Test `{}`:", name);
    table.add_row(vec![
        Cell::new("TEST").fg(Color::Green),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Verification").fg(Color::Green),
    ]);
    for stmt in body {
        add_statement(table, stmt, level + 1);
    }
}

/// Translates a semantic expression into a readable English string.
///
/// This exists to flatten recursive expression trees (like `AnalyzedExprKind::BinOp`)
/// into linear, human-readable strings. Unlike a standard `Debug` representation
/// which outputs nested structs, this formats operations in a pseudo-code style
/// that is immediately recognizable to developers.
#[allow(dead_code)]
pub(crate) fn tell_expr(expr: &AnalyzedExpr) -> String {
    let mut buf = String::with_capacity(32);
    tell_expr_into(expr, &mut buf);
    buf
}

pub(crate) fn tell_expr_into(expr: &AnalyzedExpr, buf: &mut String) {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => {
            let _ = write!(buf, "\"{}\"", s);
        }
        AnalyzedExprKind::NumberLiteral(n) => {
            let _ = write!(buf, "{}", n);
        }
        AnalyzedExprKind::BooleanLiteral(b) => {
            let _ = write!(buf, "{}", b);
        }
        AnalyzedExprKind::Variable(name) => {
            let _ = write!(buf, "`{}`", name);
        }
        AnalyzedExprKind::VerbCall { verb, args } => tell_verb_call_into(verb, args, buf),
        AnalyzedExprKind::BinOp { left, op, right } => {
            buf.push('(');
            tell_expr_into(left, buf);
            let _ = write!(buf, " {:?} ", op);
            tell_expr_into(right, buf);
            buf.push(')');
        }
        AnalyzedExprKind::UnaryOp { op, operand } => {
            let _ = write!(buf, "({:?} ", op);
            tell_expr_into(operand, buf);
            buf.push(')');
        }
        AnalyzedExprKind::Range {
            start,
            end,
            inclusive,
        } => {
            tell_expr_into(start, buf);
            let range_op = if *inclusive { "..=" } else { ".." };
            buf.push_str(range_op);
            tell_expr_into(end, buf);
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => tell_array_literal_into(exprs, buf),
        AnalyzedExprKind::Some(e) => {
            buf.push_str("Some(");
            tell_expr_into(e, buf);
            buf.push(')');
        }
        AnalyzedExprKind::None => buf.push_str("None"),
        AnalyzedExprKind::Ok(e) => {
            buf.push_str("Ok(");
            tell_expr_into(e, buf);
            buf.push(')');
        }
        AnalyzedExprKind::Err(e) => {
            buf.push_str("Err(");
            tell_expr_into(e, buf);
            buf.push(')');
        }
        AnalyzedExprKind::Unwrap(e) => {
            tell_expr_into(e, buf);
            buf.push('!');
        }
        AnalyzedExprKind::Try(e) => {
            tell_expr_into(e, buf);
            buf.push('?');
        }
        AnalyzedExprKind::IndexAccess { array, index } => {
            tell_expr_into(array, buf);
            buf.push('[');
            tell_expr_into(index, buf);
            buf.push(']');
        }
        AnalyzedExprKind::FunctionCall { func, args } => tell_function_call_into(func, args, buf),
        AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } => tell_method_call_into(receiver, method, args, buf),
        AnalyzedExprKind::StructInstantiation {
            type_name,
            fields,
            args,
        } => tell_struct_instantiation_into(type_name, fields, args, buf),
        AnalyzedExprKind::Lambda {
            params,
            body,
            capture_mode,
        } => tell_lambda_into(params, body, capture_mode, buf),
        AnalyzedExprKind::CollectionNew { collection_type } => {
            let _ = write!(buf, "{}::new()", collection_type);
        }
        AnalyzedExprKind::Assert { condition } => {
            buf.push_str("assert(");
            tell_expr_into(condition, buf);
            buf.push(')');
        }
        AnalyzedExprKind::AssertEq { left, right } => {
            buf.push_str("assert_eq(");
            tell_expr_into(left, buf);
            buf.push_str(", ");
            tell_expr_into(right, buf);
            buf.push(')');
        }
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            tell_expr_into(owner, buf);
            let _ = write!(buf, ".{}", property);
        }
    }
}

fn tell_verb_call_into(verb: &str, args: &[AnalyzedExpr], buf: &mut String) {
    let _ = write!(buf, "{}(", verb);
    format_exprs(args, buf);
    buf.push(')');
}

fn tell_array_literal_into(exprs: &[AnalyzedExpr], buf: &mut String) {
    buf.push('[');
    format_exprs(exprs, buf);
    buf.push(']');
}

fn tell_function_call_into(func: &str, args: &[AnalyzedExpr], buf: &mut String) {
    let _ = write!(buf, "{}(", func);
    format_exprs(args, buf);
    buf.push(')');
}

fn tell_method_call_into(
    receiver: &AnalyzedExpr,
    method: &str,
    args: &[AnalyzedExpr],
    buf: &mut String,
) {
    tell_expr_into(receiver, buf);
    let _ = write!(buf, ".{}(", method);
    format_exprs(args, buf);
    buf.push(')');
}

fn tell_struct_instantiation_into(
    type_name: &str,
    fields: &[smol_str::SmolStr],
    args: &[AnalyzedExpr],
    buf: &mut String,
) {
    let _ = write!(buf, "{} {{ ", type_name);
    for (i, (f, a)) in fields.iter().zip(args.iter()).enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        let _ = write!(buf, "{}: ", f);
        tell_expr_into(a, buf);
    }
    buf.push_str(" }");
}

fn tell_lambda_into(
    params: &[smol_str::SmolStr],
    body: &AnalyzedExpr,
    capture_mode: &CaptureMode,
    buf: &mut String,
) {
    let mode = match capture_mode {
        CaptureMode::Borrow => "",
        CaptureMode::Move => "move ",
    };
    let _ = write!(buf, "{}|{}| ", mode, params.join(", "));
    tell_expr_into(body, buf);
}

/// Converts a semantic type into a familiar Rust-like type signature string.
///
/// While ΓΛΩΣΣΑ uses Greek terminology internally (e.g., `ἀριθμός`, `λίστη`),
/// the Scroll of Logic translates these into conventional programming type names
/// (e.g., `Number`, `[Type]`) to help developers map the Greek concepts to
/// concepts they already understand.
fn tell_type(ty: &GlossaType) -> String {
    let mut buf = String::with_capacity(32);
    tell_type_into(ty, &mut buf);
    buf
}

fn tell_type_into(ty: &GlossaType, buf: &mut String) {
    match ty {
        GlossaType::Number => buf.push_str("Number"),
        GlossaType::String => buf.push_str("String"),
        GlossaType::Boolean => buf.push_str("Bool"),
        GlossaType::List(inner) => {
            buf.push('[');
            tell_type_into(inner, buf);
            buf.push(']');
        }
        GlossaType::Set(inner) => {
            buf.push_str("Set<");
            tell_type_into(inner, buf);
            buf.push('>');
        }
        GlossaType::Map(k, v) => {
            buf.push_str("Map<");
            tell_type_into(k, buf);
            buf.push_str(", ");
            tell_type_into(v, buf);
            buf.push('>');
        }
        GlossaType::Option(inner) => {
            buf.push_str("Option<");
            tell_type_into(inner, buf);
            buf.push('>');
        }
        GlossaType::Result(ok, err) => {
            buf.push_str("Result<");
            tell_type_into(ok, buf);
            buf.push_str(", ");
            tell_type_into(err, buf);
            buf.push('>');
        }
        GlossaType::Struct { name, .. } => buf.push_str(name),
        GlossaType::Function { params, returns } => {
            buf.push_str("Fn(");
            format_types(params, buf);
            buf.push_str(") -> ");
            tell_type_into(returns, buf);
        }
        GlossaType::Unit => buf.push_str("()"),
        GlossaType::Unknown => buf.push('?'),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    #[test]
    fn test_bard_basic() {
        let source = "ξ πέντε ἔστω.";
        let ast = parse(source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let tale = tell_tale(&analyzed);

        // Check for table content instead of full sentence
        assert!(tale.contains("BIND"));
        assert!(tale.contains("Let `ξ` be 5"));
    }

    #[test]
    fn test_bard_print() {
        let source = "«χαῖρε» λέγε.";
        let ast = parse(source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let tale = tell_tale(&analyzed);

        assert!(tale.contains("PRINT"));
        assert!(tale.contains("Proclaim: \"χαῖρε\""));
    }

    #[test]
    fn test_tell_expr_all_variants() {
        use crate::morphology::lexicon::{BinaryOp, UnaryOp};
        use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};

        let kinds = vec![
            AnalyzedExprKind::StringLiteral("test".to_string()),
            AnalyzedExprKind::NumberLiteral(42),
            AnalyzedExprKind::BooleanLiteral(true),
            AnalyzedExprKind::Variable("var".to_string().into()),
            AnalyzedExprKind::PropertyAccess {
                owner: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable("obj".to_string().into()),
                    glossa_type: GlossaType::Unknown,
                }),
                property: "prop".to_string().into(),
            },
            AnalyzedExprKind::VerbCall {
                verb: "call".to_string().into(),
                args: vec![],
            },
            AnalyzedExprKind::BinOp {
                left: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }),
                op: BinaryOp::Add,
                right: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(2),
                    glossa_type: GlossaType::Number,
                }),
            },
            AnalyzedExprKind::UnaryOp {
                op: UnaryOp::Neg,
                operand: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }),
            },
            AnalyzedExprKind::Range {
                start: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }),
                end: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(10),
                    glossa_type: GlossaType::Number,
                }),
                inclusive: true,
            },
            AnalyzedExprKind::ArrayLiteral(vec![]),
            AnalyzedExprKind::Some(Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            })),
            AnalyzedExprKind::None,
            AnalyzedExprKind::Ok(Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            })),
            AnalyzedExprKind::Err(Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::StringLiteral("err".to_string()),
                glossa_type: GlossaType::String,
            })),
            AnalyzedExprKind::Unwrap(Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("x".to_string().into()),
                glossa_type: GlossaType::Unknown,
            })),
            AnalyzedExprKind::Try(Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("x".to_string().into()),
                glossa_type: GlossaType::Unknown,
            })),
            AnalyzedExprKind::IndexAccess {
                array: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable("arr".to_string().into()),
                    glossa_type: GlossaType::Unknown,
                }),
                index: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(0),
                    glossa_type: GlossaType::Number,
                }),
            },
            AnalyzedExprKind::FunctionCall {
                func: "fn".to_string().into(),
                args: vec![],
            },
            AnalyzedExprKind::MethodCall {
                receiver: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable("obj".to_string().into()),
                    glossa_type: GlossaType::Unknown,
                }),
                method: "meth".to_string().into(),
                args: vec![],
            },
            AnalyzedExprKind::StructInstantiation {
                type_name: "Type".to_string().into(),
                fields: vec![],
                args: vec![],
            },
            AnalyzedExprKind::Lambda {
                params: vec!["p".into()],
                body: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }),
                capture_mode: crate::semantic::CaptureMode::Borrow,
            },
            AnalyzedExprKind::CollectionNew {
                collection_type: GlossaType::Unknown.to_string(),
            },
            AnalyzedExprKind::Assert {
                condition: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                }),
            },
            AnalyzedExprKind::AssertEq {
                left: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }),
                right: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }),
            },
        ];

        for kind in kinds {
            let expr = AnalyzedExpr {
                expr: kind,
                glossa_type: GlossaType::Unknown,
            };
            let formatted = tell_expr(&expr);
            assert!(!formatted.is_empty());
        }
    }

    #[test]
    fn test_tell_statement_all_variants() {
        use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType};

        let dummy_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        };

        let stmts = vec![
            AnalyzedStatement::Query(vec![dummy_expr.clone()]),
            AnalyzedStatement::For {
                variable: "i".to_string().into(),
                iterator: Box::new(dummy_expr.clone()),
                body: vec![],
            },
            AnalyzedStatement::Match {
                scrutinee: Box::new(dummy_expr.clone()),
                arms: vec![(dummy_expr.clone(), vec![])],
            },
            AnalyzedStatement::Break,
            AnalyzedStatement::Continue,
            AnalyzedStatement::Return {
                value: Some(Box::new(dummy_expr.clone())),
            },
            AnalyzedStatement::Return { value: None },
        ];

        for stmt in stmts {
            let mut table = comfy_table::Table::new();
            add_statement(&mut table, &stmt, 0);
            assert!(!table.is_empty());
        }
    }
}
