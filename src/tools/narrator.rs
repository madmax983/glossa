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
/// ⚡ Bolt Optimization: Avoids intermediate `Vec<String>` allocation.
fn join_exprs(exprs: &[AnalyzedExpr]) -> String {
    let mut buf = String::with_capacity(exprs.len() * 16);
    for (i, expr) in exprs.iter().enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        buf.push_str(&tell_expr(expr));
    }
    buf
}

/// ⚡ Bolt Optimization: Avoids intermediate `Vec<String>` allocation.
fn join_types(types: &[GlossaType]) -> String {
    let mut buf = String::with_capacity(types.len() * 8);
    for (i, ty) in types.iter().enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        buf.push_str(&tell_type(ty));
    }
    buf
}

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
    let script = format!("Let `{}` be {}.", name, tell_expr(value));
    let notes = if mutable { "Mutable" } else { "Immutable" };
    table.add_row(vec![
        Cell::new("BIND").fg(Color::Blue),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new(notes).fg(if mutable { Color::Red } else { Color::Green }),
    ]);
}

fn add_assignment(table: &mut Table, prefix: &str, name: &str, value: &AnalyzedExpr) {
    let script = format!("Update `{}` to {}.", name, tell_expr(value));
    table.add_row(vec![
        Cell::new("SET").fg(Color::Yellow),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Mutation").fg(Color::Red),
    ]);
}

fn add_print(table: &mut Table, prefix: &str, exprs: &[AnalyzedExpr]) {
    let expr_strs = join_exprs(exprs);
    let script = format!("Proclaim: {}", expr_strs);
    table.add_row(vec![
        Cell::new("PRINT").fg(Color::Green),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("I/O").fg(Color::Cyan),
    ]);
}

fn add_expression(table: &mut Table, prefix: &str, exprs: &[AnalyzedExpr]) {
    let expr_strs = join_exprs(exprs);
    let script = format!("Do: {}", expr_strs);
    table.add_row(vec![
        Cell::new("EXPR").fg(Color::DarkGrey),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Side Effect").fg(Color::DarkGrey),
    ]);
}

fn add_query(table: &mut Table, prefix: &str, exprs: &[AnalyzedExpr]) {
    let expr_strs = join_exprs(exprs);
    let script = format!("Query oracle: {}", expr_strs);
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
    let script = format!("If {} is true, then:", tell_expr(condition));
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
    let script = format!("While {} holds true:", tell_expr(condition));
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
    let script = format!("For each `{}` in {}:", variable, tell_expr(iterator));
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
    let script = format!("Match on {}:", tell_expr(scrutinee));
    table.add_row(vec![
        Cell::new("MATCH").fg(Color::Magenta),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new("Pattern").fg(Color::Magenta),
    ]);
    for (pat, body) in arms {
        let case_script = format!("Case {}:", tell_expr(pat));
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
        format!("Return {}.", tell_expr(v))
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
    let params_str: Vec<String> = params
        .iter()
        .map(|(n, t)| {
            let type_str = t.as_ref().map(tell_type).unwrap_or("unknown".to_string());
            format!("{}: {}", n, type_str)
        })
        .collect();
    let ret_str = return_type
        .as_ref()
        .map(tell_type)
        .unwrap_or("Nothing".to_string());

    let script = format!(
        "Define `{}` ({}) -> {}:",
        name,
        params_str.join(", "),
        ret_str
    );
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
    let fields_str: Vec<String> = fields
        .iter()
        .map(|(n, t)| format!("{}: {}", n, tell_type(t)))
        .collect();
    let script = format!("Struct `{}` {{ {} }}", name, fields_str.join(", "));
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
fn tell_expr(expr: &AnalyzedExpr) -> String {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => format!("\"{}\"", s),
        AnalyzedExprKind::NumberLiteral(n) => format!("{}", n),
        AnalyzedExprKind::BooleanLiteral(b) => format!("{}", b),
        AnalyzedExprKind::Variable(name) => format!("`{}`", name),
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            format!("{}.{}", tell_expr(owner), property)
        }
        AnalyzedExprKind::VerbCall { verb, args } => tell_verb_call(verb, args),
        AnalyzedExprKind::BinOp { left, op, right } => {
            format!("({} {:?} {})", tell_expr(left), op, tell_expr(right))
        }
        AnalyzedExprKind::UnaryOp { op, operand } => {
            format!("({:?} {})", op, tell_expr(operand))
        }
        AnalyzedExprKind::Range {
            start,
            end,
            inclusive,
        } => {
            let range_op = if *inclusive { "..=" } else { ".." };
            format!("{}{}{}", tell_expr(start), range_op, tell_expr(end))
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => tell_array_literal(exprs),
        AnalyzedExprKind::Some(e) => format!("Some({})", tell_expr(e)),
        AnalyzedExprKind::None => "None".to_string(),
        AnalyzedExprKind::Ok(e) => format!("Ok({})", tell_expr(e)),
        AnalyzedExprKind::Err(e) => format!("Err({})", tell_expr(e)),
        AnalyzedExprKind::Unwrap(e) => format!("{}!", tell_expr(e)),
        AnalyzedExprKind::Try(e) => format!("{}?", tell_expr(e)),
        AnalyzedExprKind::IndexAccess { array, index } => {
            format!("{}[{}]", tell_expr(array), tell_expr(index))
        }
        AnalyzedExprKind::FunctionCall { func, args } => tell_function_call(func, args),
        AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } => tell_method_call(receiver, method, args),
        AnalyzedExprKind::TraitMethodCall {
            receiver,
            trait_name,
            method_name,
            args,
        } => tell_trait_method_call(receiver, trait_name, method_name, args),
        AnalyzedExprKind::StructInstantiation {
            type_name,
            fields,
            args,
        } => tell_struct_instantiation(type_name, fields, args),
        AnalyzedExprKind::Lambda {
            params,
            body,
            capture_mode,
        } => tell_lambda(params, body, capture_mode),
        AnalyzedExprKind::CollectionNew { collection_type } => {
            format!("{}::new()", collection_type)
        }
        AnalyzedExprKind::Assert { condition } => {
            format!("assert({})", tell_expr(condition))
        }
        AnalyzedExprKind::AssertEq { left, right } => {
            format!("assert_eq({}, {})", tell_expr(left), tell_expr(right))
        }
    }
}

fn tell_verb_call(verb: &str, args: &[AnalyzedExpr]) -> String {
    let args_str = join_exprs(args);
    format!("{}({})", verb, args_str)
}

fn tell_array_literal(exprs: &[AnalyzedExpr]) -> String {
    let expr_strs = join_exprs(exprs);
    format!("[{}]", expr_strs)
}

fn tell_function_call(func: &str, args: &[AnalyzedExpr]) -> String {
    let args_str = join_exprs(args);
    format!("{}({})", func, args_str)
}

fn tell_method_call(receiver: &AnalyzedExpr, method: &str, args: &[AnalyzedExpr]) -> String {
    let args_str = join_exprs(args);
    format!("{}.{}({})", tell_expr(receiver), method, args_str)
}

fn tell_trait_method_call(
    receiver: &AnalyzedExpr,
    trait_name: &str,
    method_name: &str,
    args: &[AnalyzedExpr],
) -> String {
    let args_str = join_exprs(args);
    format!(
        "{} as {}::{}({})",
        tell_expr(receiver),
        trait_name,
        method_name,
        args_str
    )
}

fn tell_struct_instantiation(
    type_name: &str,
    fields: &[smol_str::SmolStr],
    args: &[AnalyzedExpr],
) -> String {
    let mut fields_args = String::with_capacity(fields.len() * 16);
    for (i, (f, a)) in fields.iter().zip(args.iter()).enumerate() {
        if i > 0 {
            fields_args.push_str(", ");
        }
        fields_args.push_str(&format!("{}: {}", f, tell_expr(a)));
    }
    format!("{} {{ {} }}", type_name, fields_args)
}

fn tell_lambda(
    params: &[smol_str::SmolStr],
    body: &AnalyzedExpr,
    capture_mode: &CaptureMode,
) -> String {
    let mode = match capture_mode {
        CaptureMode::Borrow => "",
        CaptureMode::Move => "move ",
        CaptureMode::Memoize => "memo ",
    };
    format!("{}|{}| {}", mode, params.join(", "), tell_expr(body))
}

/// Converts a semantic type into a familiar Rust-like type signature string.
///
/// While ΓΛΩΣΣΑ uses Greek terminology internally (e.g., `ἀριθμός`, `λίστη`),
/// the Scroll of Logic translates these into conventional programming type names
/// (e.g., `Number`, `[Type]`) to help developers map the Greek concepts to
/// concepts they already understand.
fn tell_type(ty: &GlossaType) -> String {
    match ty {
        GlossaType::Number => "Number".to_string(),
        GlossaType::String => "String".to_string(),
        GlossaType::Boolean => "Bool".to_string(),
        GlossaType::List(inner) => format!("[{}]", tell_type(inner)),
        GlossaType::Set(inner) => format!("Set<{}>", tell_type(inner)),
        GlossaType::Map(k, v) => format!("Map<{}, {}>", tell_type(k), tell_type(v)),
        GlossaType::Option(inner) => format!("Option<{}>", tell_type(inner)),
        GlossaType::Result(ok, err) => format!("Result<{}, {}>", tell_type(ok), tell_type(err)),
        GlossaType::Struct { name, .. } => name.to_string(),
        GlossaType::Function { params, returns } => {
            let params_str = join_types(params);
            format!("Fn({}) -> {}", params_str, tell_type(returns))
        }
        GlossaType::Unit => "()".to_string(),
        GlossaType::Unknown => "?".to_string(),
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
}
