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

/// Tells the tale of the program in English.
///
/// This function translates the semantic meaning of the program into a readable English narrative.
/// It acts as the entry point for the "Bard" tool.
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

fn indent(level: usize) -> String {
    "  ".repeat(level)
}

fn add_statement(table: &mut Table, stmt: &AnalyzedStatement, level: usize) {
    let prefix = indent(level);

    let (act, script, notes) = describe_statement(stmt);
    if act == "EMPTY" {
        return;
    }

    let act_color = match act.as_str() {
        "BIND" => Color::Blue,
        "SET" => Color::Yellow,
        "PRINT" => Color::Green,
        "EXPR" => Color::DarkGrey,
        "QUERY" => Color::Magenta,
        "IF" | "WHILE" | "FOR" | "MATCH" => Color::Magenta,
        "BREAK" | "RET" => Color::Red,
        "CONT" => Color::Yellow,
        "DEF" | "TYPE" | "TRAIT" | "IMPL" => Color::Cyan,
        "TEST" => Color::Yellow,
        _ => Color::White,
    };

    let notes_color = match act.as_str() {
        "BIND" => {
            if notes == "Mutable" {
                Color::Red
            } else {
                Color::Green
            }
        }
        "SET" => Color::Red,
        "PRINT" => Color::Cyan,
        "EXPR" => Color::DarkGrey,
        "QUERY" => Color::Yellow,
        "IF" | "WHILE" | "FOR" | "MATCH" => Color::Magenta,
        "BREAK" | "RET" => Color::Red,
        "CONT" => Color::Yellow,
        "DEF" | "IMPL" => Color::Cyan,
        "TYPE" => Color::Cyan,
        "TRAIT" => Color::Cyan,
        "TEST" => Color::Yellow,
        _ => Color::White,
    };

    table.add_row(vec![
        Cell::new(&act).fg(act_color),
        Cell::new(format!("{}{}", prefix, script)),
        Cell::new(&notes).fg(notes_color),
    ]);

    // Process nested structures
    match stmt {
        AnalyzedStatement::If {
            then_body,
            else_body,
            ..
        } => {
            for nested_stmt in then_body {
                add_statement(table, nested_stmt, level + 1);
            }
            if let Some(else_stmts) = else_body {
                table.add_row(vec![
                    Cell::new("ELSE").fg(Color::Magenta),
                    Cell::new(format!("{}Otherwise:", prefix)),
                    Cell::new("Branch").fg(Color::Magenta),
                ]);
                for nested_stmt in else_stmts {
                    add_statement(table, nested_stmt, level + 1);
                }
            }
        }
        AnalyzedStatement::While { body, .. } | AnalyzedStatement::For { body, .. } => {
            for nested_stmt in body {
                add_statement(table, nested_stmt, level + 1);
            }
        }
        AnalyzedStatement::Match { arms, .. } => {
            for (pattern, body) in arms {
                table.add_row(vec![
                    Cell::new("ARM").fg(Color::DarkGrey),
                    Cell::new(format!("{}  If {}:", prefix, tell_expr(pattern))),
                    Cell::new("Case").fg(Color::DarkGrey),
                ]);
                for nested_stmt in body {
                    add_statement(table, nested_stmt, level + 2);
                }
            }
        }
        AnalyzedStatement::FunctionDef { body, .. } => {
            for nested_stmt in body {
                add_statement(table, nested_stmt, level + 1);
            }
        }
        AnalyzedStatement::TypeDefinition { fields, .. } => {
            for (name, ty) in fields {
                table.add_row(vec![
                    Cell::new("FIELD").fg(Color::DarkGrey),
                    Cell::new(format!("{}  `{}` of type {}", prefix, name, tell_type(ty))),
                    Cell::new("Data").fg(Color::DarkGrey),
                ]);
            }
        }
        AnalyzedStatement::TraitDefinition { methods, .. } => {
            for method in methods {
                table.add_row(vec![
                    Cell::new("REQ").fg(Color::DarkGrey),
                    Cell::new(format!("{}  Requires method `{}`", prefix, method.name)),
                    Cell::new("Contract").fg(Color::DarkGrey),
                ]);
            }
        }
        AnalyzedStatement::TraitImplementation { methods, .. } => {
            for func in methods {
                table.add_row(vec![
                    Cell::new("METH").fg(Color::Cyan),
                    Cell::new(format!("{}  Method `{}`", prefix, func.name)),
                    Cell::new("Function").fg(Color::Cyan),
                ]);
                if let Some(body) = &func.body {
                    for nested_stmt in body {
                        add_statement(table, nested_stmt, level + 2);
                    }
                }
            }
        }
        AnalyzedStatement::TestDeclaration { body, .. } => {
            for nested_stmt in body {
                add_statement(table, nested_stmt, level + 1);
            }
        }
        _ => {}
    }
}
pub fn tell_expr(expr: &AnalyzedExpr) -> String {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => format!("\"{}\"", s),
        AnalyzedExprKind::NumberLiteral(n) => format!("{}", n),
        AnalyzedExprKind::BooleanLiteral(b) => format!("{}", b),
        AnalyzedExprKind::Variable(name) => format!("`{}`", name),
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

pub fn tell_type(ty: &GlossaType) -> String {
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

pub fn describe_statement(stmt: &AnalyzedStatement) -> (String, String, String) {
    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            let script = format!("Let `{}` be {}.", name, tell_expr(value));
            let notes = if *mutable { "Mutable" } else { "Immutable" };
            ("BIND".to_string(), script, notes.to_string())
        }
        AnalyzedStatement::Assignment { name, value } => {
            let script = format!("Update `{}` to {}.", name, tell_expr(value));
            ("SET".to_string(), script, "Mutation".to_string())
        }
        AnalyzedStatement::Print(exprs) => {
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            let script = format!("Proclaim: {}", expr_strs.join(", "));
            ("PRINT".to_string(), script, "I/O".to_string())
        }
        AnalyzedStatement::Expression(exprs) => {
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            let script = format!("Do: {}", expr_strs.join(", "));
            ("EXPR".to_string(), script, "Side Effect".to_string())
        }
        AnalyzedStatement::Query(exprs) => {
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            let script = format!("Query oracle: {}", expr_strs.join(", "));
            ("QUERY".to_string(), script, "Debug".to_string())
        }
        AnalyzedStatement::If { condition, .. } => {
            let script = format!("If {} is true, then:", tell_expr(condition));
            ("IF".to_string(), script, "Branch".to_string())
        }
        AnalyzedStatement::While { condition, .. } => {
            let script = format!("While {} holds true:", tell_expr(condition));
            ("WHILE".to_string(), script, "Loop".to_string())
        }
        AnalyzedStatement::For {
            variable, iterator, ..
        } => {
            let script = format!("For each `{}` in {}:", variable, tell_expr(iterator));
            ("FOR".to_string(), script, "Iteration".to_string())
        }
        AnalyzedStatement::Match { scrutinee, .. } => {
            let script = format!("Match {} against:", tell_expr(scrutinee));
            ("MATCH".to_string(), script, "Pattern".to_string())
        }
        AnalyzedStatement::Break => (
            "BREAK".to_string(),
            "Break from the loop.".to_string(),
            "Control Flow".to_string(),
        ),
        AnalyzedStatement::Continue => (
            "CONT".to_string(),
            "Continue to the next iteration.".to_string(),
            "Control Flow".to_string(),
        ),
        AnalyzedStatement::Return { value } => {
            let script = if let Some(e) = value {
                format!("Return {}.", tell_expr(e))
            } else {
                "Return.".to_string()
            };
            ("RET".to_string(), script, "Control Flow".to_string())
        }
        AnalyzedStatement::FunctionDef {
            name,
            params,
            return_type,
            ..
        } => {
            let args: Vec<String> = params.iter().map(|(p, _)| format!("`{}`", p)).collect();
            let returns = if let Some(ret) = return_type {
                format!(" returning {}", tell_type(ret))
            } else {
                "".to_string()
            };
            let script = format!(
                "Define action `{}` taking ({}){}",
                name,
                args.join(", "),
                returns
            );
            ("DEF".to_string(), script, "Function".to_string())
        }
        AnalyzedStatement::TypeDefinition { name, .. } => {
            let script = format!("Define structure `{}`:", name);
            ("TYPE".to_string(), script, "Struct".to_string())
        }
        AnalyzedStatement::TraitDefinition { name, .. } => {
            let script = format!("Define trait `{}`:", name);
            ("TRAIT".to_string(), script, "Interface".to_string())
        }
        AnalyzedStatement::TraitImplementation {
            trait_name,
            type_name,
            ..
        } => {
            let script = format!("Implement trait `{}` for `{}`:", trait_name, type_name);
            ("IMPL".to_string(), script, "Implementation".to_string())
        }
        AnalyzedStatement::TestDeclaration { name, .. } => {
            let script = format!("Declare test `{}`:", name);
            ("TEST".to_string(), script, "Verification".to_string())
        }
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
