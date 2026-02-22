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
use crossterm::style::Stylize;

#[derive(Copy, Clone)]
struct NarratorTheme {
    binding: Color,
    assignment: Color,
    print: Color,
    expr: Color,
    query: Color,
    control: Color,
    structure: Color,
}

impl Default for NarratorTheme {
    fn default() -> Self {
        Self {
            binding: Color::Cyan,
            assignment: Color::Yellow,
            print: Color::Green,
            expr: Color::DarkGrey,
            query: Color::Magenta,
            control: Color::Red,
            structure: Color::Blue,
        }
    }
}

/// Tells the tale of the program in English.
///
/// This function translates the semantic meaning of the program into a readable English narrative.
/// It acts as the entry point for the "Bard" tool.
pub fn tell_tale(program: &AnalyzedProgram) -> String {
    let mut table = Table::new();
    let theme = NarratorTheme::default();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Act")
                .add_attribute(Attribute::Bold)
                .fg(theme.binding),
            Cell::new("The Scroll of Logic")
                .add_attribute(Attribute::Bold)
                .fg(theme.assignment),
            Cell::new("Notes")
                .add_attribute(Attribute::Bold)
                .fg(theme.query),
        ]);

    for stmt in &program.statements {
        add_statement(&mut table, stmt, 0, theme);
    }

    // Add a footer
    table.add_row(vec![
        Cell::new("FIN").fg(theme.expr),
        Cell::new("...and thus the ritual is complete.")
            .fg(theme.expr)
            .add_attribute(Attribute::Italic),
        Cell::new("").fg(theme.expr),
    ]);

    table.to_string()
}

fn indent(level: usize) -> String {
    "  ".repeat(level)
}

fn add_statement(table: &mut Table, stmt: &AnalyzedStatement, level: usize, theme: NarratorTheme) {
    let prefix = indent(level);

    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            let script = format!(
                "Let {} be {}.",
                format!("`{}`", name).white().italic(),
                tell_expr(value)
            );
            let notes = if *mutable { "Mutable" } else { "Immutable" };
            table.add_row(vec![
                Cell::new("BIND 📝").fg(theme.binding),
                Cell::new(format!("{}{}", prefix, script)),
                Cell::new(notes).fg(if *mutable { theme.control } else { theme.print }),
            ]);
        }
        AnalyzedStatement::Assignment { name, value } => {
            let script = format!(
                "Update {} to {}.",
                format!("`{}`", name).white().italic(),
                tell_expr(value)
            );
            table.add_row(vec![
                Cell::new("SET ✏️").fg(theme.assignment),
                Cell::new(format!("{}{}", prefix, script)),
                Cell::new("Mutation").fg(theme.control),
            ]);
        }
        AnalyzedStatement::Print(exprs) => {
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            let script = format!("Proclaim: {}", expr_strs.join(", "));
            table.add_row(vec![
                Cell::new("PRINT 📢").fg(theme.print),
                Cell::new(format!("{}{}", prefix, script)),
                Cell::new("I/O").fg(theme.binding),
            ]);
        }
        AnalyzedStatement::Expression(exprs) => {
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            let script = format!("Do: {}", expr_strs.join(", "));
            table.add_row(vec![
                Cell::new("EXPR ⚡").fg(theme.expr),
                Cell::new(format!("{}{}", prefix, script)),
                Cell::new("Side Effect").fg(theme.expr),
            ]);
        }
        AnalyzedStatement::Query(exprs) => {
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            let script = format!("Query oracle: {}", expr_strs.join(", "));
            table.add_row(vec![
                Cell::new("QUERY 🔮").fg(theme.query),
                Cell::new(format!("{}{}", prefix, script)),
                Cell::new("Debug").fg(theme.assignment),
            ]);
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            let script = format!("If {} is true, then:", tell_expr(condition));
            table.add_row(vec![
                Cell::new("IF 🔀").fg(theme.query),
                Cell::new(format!("{}{}", prefix, script)),
                Cell::new("Branch").fg(theme.query),
            ]);

            for stmt in then_body {
                add_statement(table, stmt, level + 1, theme);
            }

            if let Some(else_stmts) = else_body {
                table.add_row(vec![
                    Cell::new("ELSE ↔️").fg(theme.query),
                    Cell::new(format!("{}Otherwise:", prefix)),
                    Cell::new("Branch").fg(theme.query),
                ]);
                for stmt in else_stmts {
                    add_statement(table, stmt, level + 1, theme);
                }
            }
        }
        AnalyzedStatement::While { condition, body } => {
            let script = format!("While {} holds true:", tell_expr(condition));
            table.add_row(vec![
                Cell::new("WHILE 🔄").fg(theme.query),
                Cell::new(format!("{}{}", prefix, script)),
                Cell::new("Loop").fg(theme.query),
            ]);
            for stmt in body {
                add_statement(table, stmt, level + 1, theme);
            }
        }
        AnalyzedStatement::For {
            variable,
            iterator,
            body,
        } => {
            let script = format!("For each `{}` in {}:", variable, tell_expr(iterator));
            table.add_row(vec![
                Cell::new("FOR 🔁").fg(theme.query),
                Cell::new(format!("{}{}", prefix, script)),
                Cell::new("Iteration").fg(theme.query),
            ]);
            for stmt in body {
                add_statement(table, stmt, level + 1, theme);
            }
        }
        AnalyzedStatement::Match { scrutinee, arms } => {
            let script = format!("Match on {}:", tell_expr(scrutinee));
            table.add_row(vec![
                Cell::new("MATCH 🔍").fg(theme.query),
                Cell::new(format!("{}{}", prefix, script)),
                Cell::new("Pattern").fg(theme.query),
            ]);
            for (pat, body) in arms {
                let case_script = format!("Case {}:", tell_expr(pat));
                table.add_row(vec![
                    Cell::new("CASE 🎯").fg(theme.structure),
                    Cell::new(format!("{}{}", indent(level + 1), case_script)),
                    Cell::new("Arm").fg(theme.structure),
                ]);
                for stmt in body {
                    add_statement(table, stmt, level + 2, theme);
                }
            }
        }
        AnalyzedStatement::Break => {
            table.add_row(vec![
                Cell::new("BREAK 🛑").fg(theme.control),
                Cell::new(format!("{}Break loop.", prefix)),
                Cell::new("Control").fg(theme.control),
            ]);
        }
        AnalyzedStatement::Continue => {
            table.add_row(vec![
                Cell::new("CONT ⏩").fg(theme.print),
                Cell::new(format!("{}Continue loop.", prefix)),
                Cell::new("Control").fg(theme.print),
            ]);
        }
        AnalyzedStatement::Return { value } => {
            let script = if let Some(v) = value {
                format!("Return {}.", tell_expr(v))
            } else {
                "Return nothing.".to_string()
            };
            table.add_row(vec![
                Cell::new("RETURN 🚪").fg(theme.assignment),
                Cell::new(format!("{}{}", prefix, script)),
                Cell::new("Exit").fg(theme.assignment),
            ]);
        }
        AnalyzedStatement::FunctionDef {
            name,
            params,
            body,
            return_type,
        } => {
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
                Cell::new("FUNC ƒ").fg(theme.binding),
                Cell::new(format!("{}{}", prefix, script)),
                Cell::new("Definition").fg(theme.binding),
            ]);
            for stmt in body {
                add_statement(table, stmt, level + 1, theme);
            }
        }
        AnalyzedStatement::TypeDefinition { name, fields } => {
            let fields_str: Vec<String> = fields
                .iter()
                .map(|(n, t)| format!("{}: {}", n, tell_type(t)))
                .collect();
            let script = format!("Struct `{}` {{ {} }}", name, fields_str.join(", "));
            table.add_row(vec![
                Cell::new("TYPE 📦").fg(theme.structure),
                Cell::new(format!("{}{}", prefix, script)),
                Cell::new("Struct").fg(theme.structure),
            ]);
        }
        AnalyzedStatement::TraitDefinition { name, methods: _ } => {
            let script = format!("Trait `{}`", name);
            table.add_row(vec![
                Cell::new("TRAIT 🏷️").fg(theme.structure),
                Cell::new(format!("{}{}", prefix, script)),
                Cell::new("Interface").fg(theme.structure),
            ]);
        }
        AnalyzedStatement::TraitImplementation {
            trait_name,
            type_name,
            methods: _,
        } => {
            let script = format!("Impl `{}` for `{}`", trait_name, type_name);
            table.add_row(vec![
                Cell::new("IMPL 🔧").fg(theme.structure),
                Cell::new(format!("{}{}", prefix, script)),
                Cell::new("Implementation").fg(theme.structure),
            ]);
        }
        AnalyzedStatement::TestDeclaration { name, body } => {
            let script = format!("Test `{}`:", name);
            table.add_row(vec![
                Cell::new("TEST 🧪").fg(theme.print),
                Cell::new(format!("{}{}", prefix, script)),
                Cell::new("Verification").fg(theme.print),
            ]);
            for stmt in body {
                add_statement(table, stmt, level + 1, theme);
            }
        }
    }
}

fn tell_expr(expr: &AnalyzedExpr) -> String {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => format!("\"{}\"", s).green().to_string(),
        AnalyzedExprKind::NumberLiteral(n) => format!("{}", n).yellow().to_string(),
        AnalyzedExprKind::BooleanLiteral(b) => format!("{}", b).cyan().to_string(),
        AnalyzedExprKind::Variable(name) => format!("`{}`", name).white().italic().to_string(),
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            format!("{}.{}", tell_expr(owner), property)
        }
        AnalyzedExprKind::VerbCall { verb, args } => {
            let args_str: Vec<String> = args.iter().map(tell_expr).collect();
            format!("{}({})", verb, args_str.join(", "))
        }
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
        AnalyzedExprKind::ArrayLiteral(exprs) => {
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            format!("[{}]", expr_strs.join(", "))
        }
        AnalyzedExprKind::Some(e) => format!("Some({})", tell_expr(e)),
        AnalyzedExprKind::None => "None".to_string(),
        AnalyzedExprKind::Ok(e) => format!("Ok({})", tell_expr(e)),
        AnalyzedExprKind::Err(e) => format!("Err({})", tell_expr(e)),
        AnalyzedExprKind::Unwrap(e) => format!("{}!", tell_expr(e)),
        AnalyzedExprKind::Try(e) => format!("{}?", tell_expr(e)),
        AnalyzedExprKind::IndexAccess { array, index } => {
            format!("{}[{}]", tell_expr(array), tell_expr(index))
        }
        AnalyzedExprKind::FunctionCall { func, args } => {
            let args_str: Vec<String> = args.iter().map(tell_expr).collect();
            format!("{}({})", func, args_str.join(", "))
        }
        AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } => {
            let args_str: Vec<String> = args.iter().map(tell_expr).collect();
            format!(
                "{}.{}({})",
                tell_expr(receiver),
                method,
                args_str.join(", ")
            )
        }
        AnalyzedExprKind::TraitMethodCall {
            receiver,
            trait_name,
            method_name,
            args,
        } => {
            let args_str: Vec<String> = args.iter().map(tell_expr).collect();
            format!(
                "{} as {}::{}({})",
                tell_expr(receiver),
                trait_name,
                method_name,
                args_str.join(", ")
            )
        }
        AnalyzedExprKind::StructInstantiation {
            type_name,
            fields,
            args,
        } => {
            let args_str: Vec<String> = args.iter().map(tell_expr).collect();
            // Zip fields and args for better display
            let fields_args: Vec<String> = fields
                .iter()
                .zip(args_str.iter())
                .map(|(f, a)| format!("{}: {}", f, a))
                .collect();
            format!("{} {{ {} }}", type_name, fields_args.join(", "))
        }
        AnalyzedExprKind::Lambda {
            params,
            body,
            capture_mode,
        } => {
            let mode = match capture_mode {
                CaptureMode::Borrow => "",
                CaptureMode::Move => "move ",
                CaptureMode::Memoize => "memo ",
            };
            format!("{}|{}| {}", mode, params.join(", "), tell_expr(body))
        }
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
            let params_str: Vec<String> = params.iter().map(tell_type).collect();
            format!("Fn({}) -> {}", params_str.join(", "), tell_type(returns))
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
        assert!(tale.contains("BIND 📝"));
        // Reconstruct expected string with colors
        let var = "`ξ`".white().italic();
        let val = "5".yellow();
        let expected = format!("Let {} be {}", var, val);
        assert!(tale.contains(&expected));
    }

    #[test]
    fn test_bard_print() {
        let source = "«χαῖρε» λέγε.";
        let ast = parse(source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let tale = tell_tale(&analyzed);

        assert!(tale.contains("PRINT 📢"));
        let val = "\"χαῖρε\"".green();
        assert!(tale.contains(&format!("Proclaim: {}", val)));
    }
}
