use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;

use crate::semantic::CaptureMode;
use crate::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType,
};

/// Tells the tale of the program in English.
///
/// This function translates the semantic meaning of the program into a readable English narrative.
pub fn tell_tale(program: &AnalyzedProgram) -> String {
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL).set_header(vec![
        Cell::new("#")
            .add_attribute(Attribute::Bold)
            .fg(Color::DarkGrey),
        Cell::new("Act")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("The Narrative")
            .add_attribute(Attribute::Bold)
            .fg(Color::Yellow),
    ]);

    for (i, stmt) in program.statements.iter().enumerate() {
        let (act, narrative) = tell_statement(stmt, 0);
        table.add_row(vec![
            Cell::new((i + 1).to_string()).fg(Color::DarkGrey),
            Cell::new(act).fg(Color::Cyan),
            Cell::new(narrative),
        ]);
    }

    format!(
        "\n{}\n{}\n{}\n",
        "📜 THE SCROLL OF LOGIC".bold().yellow().underlined(),
        table,
        "...and thus the ritual is complete.".italic().dim()
    )
}

fn indent(level: usize) -> String {
    "  ".repeat(level)
}

fn tell_statement(stmt: &AnalyzedStatement, level: usize) -> (String, String) {
    let prefix = indent(level);
    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            let act = "Declaration".to_string();
            let mutability = if *mutable { "mutable " } else { "" };
            let narrative = format!(
                "{}Let there be a {}variable named {} with the value {}.",
                prefix,
                mutability,
                name.clone().green().bold(),
                tell_expr(value)
            );
            (act, narrative)
        }
        AnalyzedStatement::Assignment { name, value } => {
            let act = "Update".to_string();
            let narrative = format!(
                "{}Update {} to become {}.",
                prefix,
                name.clone().green().bold(),
                tell_expr(value)
            );
            (act, narrative)
        }
        AnalyzedStatement::Print(exprs) => {
            let act = "Proclamation".to_string();
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            let narrative = format!("{}Proclaim to the world: {}.", prefix, expr_strs.join(", "));
            (act, narrative)
        }
        AnalyzedStatement::Expression(exprs) => {
            let act = "Action".to_string();
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            let narrative = format!("{}Perform the following: {}.", prefix, expr_strs.join(", "));
            (act, narrative)
        }
        AnalyzedStatement::Query(exprs) => {
            let act = "Query".to_string();
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            let narrative = format!(
                "{}Query the oracle about: {}.",
                prefix,
                expr_strs.join(", ")
            );
            (act, narrative)
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            let act = "Decision".to_string();
            let mut s = format!(
                "{}If it is true that {}, then:\n",
                prefix,
                tell_expr(condition)
            );
            for stmt in then_body {
                let (_, narrative) = tell_statement(stmt, level + 1);
                s.push_str(&narrative);
                s.push('\n');
            }
            if let Some(else_stmts) = else_body {
                s.push_str(&format!("{}Otherwise:\n", prefix));
                for stmt in else_stmts {
                    let (_, narrative) = tell_statement(stmt, level + 1);
                    s.push_str(&narrative);
                    s.push('\n');
                }
            }
            // Remove trailing newline for cleaner cell
            if s.ends_with('\n') {
                s.pop();
            }
            (act, s)
        }
        AnalyzedStatement::While { condition, body } => {
            let act = "Loop".to_string();
            let mut s = format!("{}As long as {}, repeat:\n", prefix, tell_expr(condition));
            for stmt in body {
                let (_, narrative) = tell_statement(stmt, level + 1);
                s.push_str(&narrative);
                s.push('\n');
            }
            if s.ends_with('\n') {
                s.pop();
            }
            (act, s)
        }
        AnalyzedStatement::For {
            variable,
            iterator,
            body,
        } => {
            let act = "Iteration".to_string();
            let mut s = format!(
                "{}For each {} found in {}, do:\n",
                prefix,
                variable.clone().green().bold(),
                tell_expr(iterator)
            );
            for stmt in body {
                let (_, narrative) = tell_statement(stmt, level + 1);
                s.push_str(&narrative);
                s.push('\n');
            }
            if s.ends_with('\n') {
                s.pop();
            }
            (act, s)
        }
        AnalyzedStatement::Match { scrutinee, arms } => {
            let act = "Matching".to_string();
            let mut s = format!(
                "{}Consider the nature of {}:\n",
                prefix,
                tell_expr(scrutinee)
            );
            for (pat, body) in arms {
                s.push_str(&format!("{}  In the case of {}:\n", prefix, tell_expr(pat)));
                for stmt in body {
                    let (_, narrative) = tell_statement(stmt, level + 2);
                    s.push_str(&narrative);
                    s.push('\n');
                }
            }
            if s.ends_with('\n') {
                s.pop();
            }
            (act, s)
        }
        AnalyzedStatement::Break => {
            let act = "Flow Control".to_string();
            let narrative = format!("{}Break the cycle.", prefix);
            (act, narrative)
        }
        AnalyzedStatement::Continue => {
            let act = "Flow Control".to_string();
            let narrative = format!("{}Continue to the next iteration.", prefix);
            (act, narrative)
        }
        AnalyzedStatement::Return { value } => {
            let act = "Return".to_string();
            let narrative = if let Some(v) = value {
                format!("{}Return with the offering {}.", prefix, tell_expr(v))
            } else {
                format!("{}Return with nothing.", prefix)
            };
            (act, narrative)
        }
        AnalyzedStatement::FunctionDef {
            name,
            params,
            body,
            return_type,
        } => {
            let act = "Ritual Definition".to_string();
            let params_str: Vec<String> = params
                .iter()
                .map(|(n, t)| {
                    let type_str = t.as_ref().map(tell_type).unwrap_or("unknown".to_string());
                    format!("{} ({})", n.clone().green().bold(), type_str)
                })
                .collect();
            let ret_str = return_type
                .as_ref()
                .map(tell_type)
                .unwrap_or("nothing".to_string());
            let mut s = format!(
                "{}Define a ritual called {} expecting [{}] which returns {}:\n",
                prefix,
                name.clone().magenta().bold(),
                params_str.join(", "),
                ret_str
            );
            for stmt in body {
                let (_, narrative) = tell_statement(stmt, level + 1);
                s.push_str(&narrative);
                s.push('\n');
            }
            if s.ends_with('\n') {
                s.pop();
            }
            (act, s)
        }
        AnalyzedStatement::TypeDefinition { name, fields } => {
            let act = "Form Definition".to_string();
            let fields_str: Vec<String> = fields
                .iter()
                .map(|(n, t)| format!("{} as {}", n.clone().green(), tell_type(t)))
                .collect();
            let narrative = format!(
                "{}Declare a new form {} with attributes: {}.",
                prefix,
                name.clone().magenta().bold(),
                fields_str.join(", ")
            );
            (act, narrative)
        }
        AnalyzedStatement::TraitDefinition { name, methods: _ } => {
            let act = "Characteristic".to_string();
            let narrative = format!(
                "{}Declare a characteristic named {}.",
                prefix,
                name.clone().magenta().bold()
            );
            (act, narrative)
        }
        AnalyzedStatement::TraitImplementation {
            trait_name,
            type_name,
            methods: _,
        } => {
            let act = "Imbue".to_string();
            let narrative = format!(
                "{}Imbue {} with the characteristic of {}.",
                prefix,
                type_name.clone().magenta().bold(),
                trait_name.clone().magenta().bold()
            );
            (act, narrative)
        }
        AnalyzedStatement::TestDeclaration { name, body } => {
            let act = "Trial".to_string();
            let mut s = format!("{}Define a trial named {}:\n", prefix, name.clone().blue());
            for stmt in body {
                let (_, narrative) = tell_statement(stmt, level + 1);
                s.push_str(&narrative);
                s.push('\n');
            }
            if s.ends_with('\n') {
                s.pop();
            }
            (act, s)
        }
    }
}

fn tell_expr(expr: &AnalyzedExpr) -> String {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => format!("\"{}\"", s.clone().yellow()),
        AnalyzedExprKind::NumberLiteral(n) => format!("{}", n.to_string().yellow()),
        AnalyzedExprKind::BooleanLiteral(b) => format!("{}", b.to_string().yellow()),
        AnalyzedExprKind::Variable(name) => format!("{}", name.clone().green()),
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            format!("the {} of {}", property.clone().green(), tell_expr(owner))
        }
        AnalyzedExprKind::VerbCall { verb, args } => {
            let args_str: Vec<String> = args.iter().map(tell_expr).collect();
            format!("{}ing [{}]", verb.clone().blue(), args_str.join(", "))
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
            let range_op = if *inclusive { "through" } else { "up to" };
            format!("from {} {} {}", tell_expr(start), range_op, tell_expr(end))
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => {
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            format!("[{}]", expr_strs.join(", "))
        }
        AnalyzedExprKind::Some(e) => format!("Some({})", tell_expr(e)),
        AnalyzedExprKind::None => "None".to_string().dim().to_string(),
        AnalyzedExprKind::Ok(e) => format!("Ok({})", tell_expr(e)),
        AnalyzedExprKind::Err(e) => format!("Err({})", tell_expr(e).red()),
        AnalyzedExprKind::Unwrap(e) => format!("unwrap({})", tell_expr(e)),
        AnalyzedExprKind::Try(e) => format!("try({})", tell_expr(e)),
        AnalyzedExprKind::IndexAccess { array, index } => {
            format!("{} at index {}", tell_expr(array), tell_expr(index))
        }
        AnalyzedExprKind::FunctionCall { func, args } => {
            let args_str: Vec<String> = args.iter().map(tell_expr).collect();
            format!("call {}({})", func.clone().blue(), args_str.join(", "))
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
                method.clone().blue(),
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
                "<{}>.{}({}, {})",
                trait_name.clone().magenta(),
                method_name.clone().blue(),
                tell_expr(receiver),
                args_str.join(", ")
            )
        }
        AnalyzedExprKind::StructInstantiation {
            type_name,
            fields,
            args,
        } => {
            let args_str: Vec<String> = args.iter().map(tell_expr).collect();
            let mut assignments = Vec::new();
            for (i, field) in fields.iter().enumerate() {
                if let Some(arg) = args_str.get(i) {
                    assignments.push(format!("{}: {}", field.clone().green(), arg));
                }
            }
            format!(
                "{} {{ {} }}",
                type_name.clone().magenta(),
                assignments.join(", ")
            )
        }
        AnalyzedExprKind::Lambda {
            params,
            body,
            capture_mode,
        } => {
            let mode = match capture_mode {
                CaptureMode::Borrow => "|",
                CaptureMode::Move => "move |",
                CaptureMode::Memoize => "memo |",
            };
            format!("{}{}| {}", mode, params.join(", "), tell_expr(body))
        }
        AnalyzedExprKind::CollectionNew { collection_type } => {
            format!("new {}", collection_type)
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
        GlossaType::String => "Text".to_string(),
        GlossaType::Boolean => "Truth".to_string(),
        GlossaType::List(inner) => format!("List<{}>", tell_type(inner)),
        GlossaType::Set(inner) => format!("Set<{}>", tell_type(inner)),
        GlossaType::Map(k, v) => format!("Map<{}, {}>", tell_type(k), tell_type(v)),
        GlossaType::Option(inner) => format!("Option<{}>", tell_type(inner)),
        GlossaType::Result(ok, err) => format!("Result<{}, {}>", tell_type(ok), tell_type(err)),
        GlossaType::Struct { name, .. } => name.to_string(),
        GlossaType::Function { params, returns } => {
            let params_str: Vec<String> = params.iter().map(tell_type).collect();
            format!("fn({}) -> {}", params_str.join(", "), tell_type(returns))
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

        // Check for table content
        assert!(tale.contains("Declaration"));
        // Check for variable name (ANSI codes might be present, so check subsequence or just existence)
        assert!(tale.contains("ξ"));
    }

    #[test]
    fn test_bard_print() {
        let source = "«χαῖρε» λέγε.";
        let ast = parse(source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let tale = tell_tale(&analyzed);

        assert!(tale.contains("Proclamation"));
        assert!(tale.contains("χαῖρε"));
    }
}
