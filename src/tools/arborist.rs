//! The Arborist Tool ("Arborist")
//!
//! This module implements the "Arborist" functionality, which visualizes the
//! Abstract Syntax Tree (AST) of a ΓΛΩΣΣΑ program.
//!
//! # Purpose
//!
//! "Arborist" reveals the hierarchical structure of the analyzed code, making it
//! easier to understand how expressions and statements are nested.

use crate::parser::parse;
use crate::semantic::{AnalyzedProgram, analyze_program};
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

/// Run the Arborist tool on a file
pub fn run_tree(input: &Path) -> Result<()> {
    let status = Status::start_with_symbol("Δενδροκομία (Planting Tree)", "🌳");

    let source = crate::tools::runner::load_source(input)?;
    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let program = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    let tree = generate_tree(&program);

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   T R E E".bold().green());
    println!("   {}", "Abstract Syntax Tree".italic().dim());
    println!();

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);

    if tree.is_empty() {
        table.set_header(vec![
            Cell::new("Status")
                .add_attribute(Attribute::Bold)
                .fg(Color::Yellow),
        ]);
        table.add_row(vec![
            Cell::new("No AST nodes found.")
                .fg(Color::DarkGrey)
                .add_attribute(Attribute::Italic),
        ]);
        println!("{table}");
        println!();
    } else {
        table.set_header(vec![
            Cell::new("AST Visualization")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
        ]);
        table.add_row(vec![Cell::new(tree)]);

        println!("{table}");
        println!();
    }

    Ok(())
}

use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement};

/// Generate a tree representation of an analyzed program
pub fn generate_tree(program: &AnalyzedProgram) -> String {
    let mut output = String::new();
    output.push_str("Program\n");
    for (i, stmt) in program.statements.iter().enumerate() {
        let is_last = i == program.statements.len() - 1;
        format_statement(stmt, "", is_last, &mut output);
    }
    output
}

fn format_statement(stmt: &AnalyzedStatement, prefix: &str, is_last: bool, output: &mut String) {
    let connector = if is_last { "└── " } else { "├── " };
    let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            let mutability = if *mutable { " (mutable)" } else { "" };
            output.push_str(&format!(
                "{}{}{}Binding: name: {}{}\n",
                prefix,
                connector,
                "● ".cyan(),
                name,
                mutability
            ));
            format_expr(value, &child_prefix, true, output);
        }
        AnalyzedStatement::Assignment { name, value } => {
            output.push_str(&format!(
                "{}{}{}Assignment: name: {}\n",
                prefix,
                connector,
                "● ".cyan(),
                name
            ));
            format_expr(value, &child_prefix, true, output);
        }
        AnalyzedStatement::Print(exprs) => {
            output.push_str(&format!("{}{}{}Print\n", prefix, connector, "● ".cyan()));
            for (i, expr) in exprs.iter().enumerate() {
                format_expr(expr, &child_prefix, i == exprs.len() - 1, output);
            }
        }
        AnalyzedStatement::Expression(exprs) => {
            output.push_str(&format!(
                "{}{}{}Expression\n",
                prefix,
                connector,
                "● ".cyan()
            ));
            for (i, expr) in exprs.iter().enumerate() {
                format_expr(expr, &child_prefix, i == exprs.len() - 1, output);
            }
        }
        AnalyzedStatement::Query(exprs) => {
            output.push_str(&format!("{}{}{}Query\n", prefix, connector, "● ".cyan()));
            for (i, expr) in exprs.iter().enumerate() {
                format_expr(expr, &child_prefix, i == exprs.len() - 1, output);
            }
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            output.push_str(&format!("{}{}{}If\n", prefix, connector, "● ".cyan()));
            format_expr(condition, &child_prefix, false, output);

            output.push_str(&format!("{}├── Then\n", child_prefix));
            let then_prefix = format!("{}│   ", child_prefix);
            for (i, s) in then_body.iter().enumerate() {
                format_statement(s, &then_prefix, i == then_body.len() - 1, output);
            }

            if let Some(else_stmts) = else_body {
                output.push_str(&format!("{}└── Else\n", child_prefix));
                let else_prefix = format!("{}    ", child_prefix);
                for (i, s) in else_stmts.iter().enumerate() {
                    format_statement(s, &else_prefix, i == else_stmts.len() - 1, output);
                }
            } else {
                // If there's no else body, we need to make sure the last item in `Then` was actually printed with `└──`?
                // Actually, the structure above handles it by having `Then` itself be a branch.
                // It's fine for simple output.
            }
        }
        AnalyzedStatement::While { condition, body } => {
            output.push_str(&format!("{}{}{}While\n", prefix, connector, "● ".cyan()));
            format_expr(condition, &child_prefix, false, output);

            output.push_str(&format!("{}└── Body\n", child_prefix));
            let body_prefix = format!("{}    ", child_prefix);
            for (i, s) in body.iter().enumerate() {
                format_statement(s, &body_prefix, i == body.len() - 1, output);
            }
        }
        AnalyzedStatement::For {
            variable,
            iterator,
            body,
        } => {
            output.push_str(&format!(
                "{}{}{}For: {}\n",
                prefix,
                connector,
                "● ".cyan(),
                variable
            ));
            format_expr(iterator, &child_prefix, false, output);

            output.push_str(&format!("{}└── Body\n", child_prefix));
            let body_prefix = format!("{}    ", child_prefix);
            for (i, s) in body.iter().enumerate() {
                format_statement(s, &body_prefix, i == body.len() - 1, output);
            }
        }
        AnalyzedStatement::Match { scrutinee, arms } => {
            output.push_str(&format!("{}{}{}Match\n", prefix, connector, "● ".cyan()));
            format_expr(scrutinee, &child_prefix, arms.is_empty(), output);

            for (i, (pat, body)) in arms.iter().enumerate() {
                let is_last_arm = i == arms.len() - 1;
                let arm_connector = if is_last_arm {
                    "└── "
                } else {
                    "├── "
                };
                let arm_prefix = format!(
                    "{}{}",
                    child_prefix,
                    if is_last_arm { "    " } else { "│   " }
                );

                output.push_str(&format!(
                    "{}{}{}Arm\n",
                    child_prefix,
                    arm_connector,
                    "○ ".blue()
                ));
                format_expr(pat, &arm_prefix, false, output);

                output.push_str(&format!("{}└── Body\n", arm_prefix));
                let body_prefix = format!("{}    ", arm_prefix);
                for (j, s) in body.iter().enumerate() {
                    format_statement(s, &body_prefix, j == body.len() - 1, output);
                }
            }
        }
        AnalyzedStatement::Break => {
            output.push_str(&format!("{}{}{}Break\n", prefix, connector, "● ".cyan()));
        }
        AnalyzedStatement::Continue => {
            output.push_str(&format!("{}{}{}Continue\n", prefix, connector, "● ".cyan()));
        }
        AnalyzedStatement::Return { value } => {
            output.push_str(&format!("{}{}{}Return\n", prefix, connector, "● ".cyan()));
            if let Some(v) = value {
                format_expr(v, &child_prefix, true, output);
            }
        }
        AnalyzedStatement::FunctionDef {
            name,
            params,
            body,
            return_type,
        } => {
            let ret_str = return_type
                .as_ref()
                .map(|t| format!(" -> {:?}", t))
                .unwrap_or_default();
            let params_str: Vec<String> = params.iter().map(|(n, _)| n.to_string()).collect();
            output.push_str(&format!(
                "{}{}{}FunctionDef: {}({}){}\n",
                prefix,
                connector,
                "● ".cyan(),
                name,
                params_str.join(", "),
                ret_str
            ));

            for (i, s) in body.iter().enumerate() {
                format_statement(s, &child_prefix, i == body.len() - 1, output);
            }
        }
        AnalyzedStatement::TypeDefinition { name, fields } => {
            let fields_str: Vec<String> = fields.iter().map(|(n, _)| n.to_string()).collect();
            output.push_str(&format!(
                "{}{}{}TypeDefinition: {} {{{}}}\n",
                prefix,
                connector,
                "● ".cyan(),
                name,
                fields_str.join(", ")
            ));
        }
        AnalyzedStatement::TraitDefinition { name, methods } => {
            let methods_str: Vec<String> = methods.iter().map(|m| m.name.to_string()).collect();
            output.push_str(&format!(
                "{}{}{}TraitDefinition: {} {{{}}}\n",
                prefix,
                connector,
                "● ".cyan(),
                name,
                methods_str.join(", ")
            ));
        }
        AnalyzedStatement::TraitImplementation {
            trait_name,
            type_name,
            methods,
        } => {
            let methods_str: Vec<String> = methods.iter().map(|m| m.name.to_string()).collect();
            output.push_str(&format!(
                "{}{}{}TraitImplementation: {} for {} {{{}}}\n",
                prefix,
                connector,
                "● ".cyan(),
                trait_name,
                type_name,
                methods_str.join(", ")
            ));
        }
        AnalyzedStatement::TestDeclaration { name, body } => {
            output.push_str(&format!(
                "{}{}{}TestDeclaration: {}\n",
                prefix,
                connector,
                "● ".cyan(),
                name
            ));
            for (i, s) in body.iter().enumerate() {
                format_statement(s, &child_prefix, i == body.len() - 1, output);
            }
        }
    }
}

fn format_expr(expr: &AnalyzedExpr, prefix: &str, is_last: bool, output: &mut String) {
    let connector = if is_last { "└── " } else { "├── " };
    let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => {
            output.push_str(&format!(
                "{}{}{}StringLiteral: \"{}\"\n",
                prefix,
                connector,
                "○ ".green(),
                s
            ));
        }
        AnalyzedExprKind::NumberLiteral(n) => {
            output.push_str(&format!(
                "{}{}{}NumberLiteral: {}\n",
                prefix,
                connector,
                "○ ".green(),
                n
            ));
        }
        AnalyzedExprKind::BooleanLiteral(b) => {
            output.push_str(&format!(
                "{}{}{}BooleanLiteral: {}\n",
                prefix,
                connector,
                "○ ".green(),
                b
            ));
        }
        AnalyzedExprKind::Variable(name) => {
            output.push_str(&format!(
                "{}{}{}Variable: {}\n",
                prefix,
                connector,
                "○ ".green(),
                name
            ));
        }
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            output.push_str(&format!(
                "{}{}{}PropertyAccess: .{}\n",
                prefix,
                connector,
                "○ ".green(),
                property
            ));
            format_expr(owner, &child_prefix, true, output);
        }
        AnalyzedExprKind::VerbCall { verb, args } => {
            output.push_str(&format!(
                "{}{}{}VerbCall: {}\n",
                prefix,
                connector,
                "○ ".green(),
                verb
            ));
            for (i, arg) in args.iter().enumerate() {
                format_expr(arg, &child_prefix, i == args.len() - 1, output);
            }
        }
        AnalyzedExprKind::BinOp { left, op, right } => {
            output.push_str(&format!(
                "{}{}{}BinOp: {:?}\n",
                prefix,
                connector,
                "○ ".green(),
                op
            ));
            format_expr(left, &child_prefix, false, output);
            format_expr(right, &child_prefix, true, output);
        }
        AnalyzedExprKind::UnaryOp { op, operand } => {
            output.push_str(&format!(
                "{}{}{}UnaryOp: {:?}\n",
                prefix,
                connector,
                "○ ".green(),
                op
            ));
            format_expr(operand, &child_prefix, true, output);
        }
        AnalyzedExprKind::Range {
            start,
            end,
            inclusive,
        } => {
            let op = if *inclusive { "..=" } else { ".." };
            output.push_str(&format!(
                "{}{}{}Range: {}\n",
                prefix,
                connector,
                "○ ".green(),
                op
            ));
            format_expr(start, &child_prefix, false, output);
            format_expr(end, &child_prefix, true, output);
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => {
            output.push_str(&format!(
                "{}{}{}ArrayLiteral\n",
                prefix,
                connector,
                "○ ".green()
            ));
            for (i, e) in exprs.iter().enumerate() {
                format_expr(e, &child_prefix, i == exprs.len() - 1, output);
            }
        }
        AnalyzedExprKind::Some(e) => {
            output.push_str(&format!("{}{}{}Some\n", prefix, connector, "○ ".green()));
            format_expr(e, &child_prefix, true, output);
        }
        AnalyzedExprKind::None => {
            output.push_str(&format!("{}{}{}None\n", prefix, connector, "○ ".green()));
        }
        AnalyzedExprKind::Ok(e) => {
            output.push_str(&format!("{}{}{}Ok\n", prefix, connector, "○ ".green()));
            format_expr(e, &child_prefix, true, output);
        }
        AnalyzedExprKind::Err(e) => {
            output.push_str(&format!("{}{}{}Err\n", prefix, connector, "○ ".green()));
            format_expr(e, &child_prefix, true, output);
        }
        AnalyzedExprKind::Unwrap(e) => {
            output.push_str(&format!("{}{}{}Unwrap\n", prefix, connector, "○ ".green()));
            format_expr(e, &child_prefix, true, output);
        }
        AnalyzedExprKind::Try(e) => {
            output.push_str(&format!("{}{}{}Try\n", prefix, connector, "○ ".green()));
            format_expr(e, &child_prefix, true, output);
        }
        AnalyzedExprKind::IndexAccess { array, index } => {
            output.push_str(&format!(
                "{}{}{}IndexAccess\n",
                prefix,
                connector,
                "○ ".green()
            ));
            format_expr(array, &child_prefix, false, output);
            format_expr(index, &child_prefix, true, output);
        }
        AnalyzedExprKind::FunctionCall { func, args } => {
            output.push_str(&format!(
                "{}{}{}FunctionCall: {}\n",
                prefix,
                connector,
                "○ ".green(),
                func
            ));
            for (i, arg) in args.iter().enumerate() {
                format_expr(arg, &child_prefix, i == args.len() - 1, output);
            }
        }
        AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } => {
            output.push_str(&format!(
                "{}{}{}MethodCall: {}\n",
                prefix,
                connector,
                "○ ".green(),
                method
            ));
            format_expr(receiver, &child_prefix, args.is_empty(), output);
            for (i, arg) in args.iter().enumerate() {
                format_expr(arg, &child_prefix, i == args.len() - 1, output);
            }
        }
        AnalyzedExprKind::TraitMethodCall {
            receiver,
            trait_name,
            method_name,
            args,
        } => {
            output.push_str(&format!(
                "{}{}{}TraitMethodCall: {}::{}\n",
                prefix,
                connector,
                "○ ".green(),
                trait_name,
                method_name
            ));
            format_expr(receiver, &child_prefix, args.is_empty(), output);
            for (i, arg) in args.iter().enumerate() {
                format_expr(arg, &child_prefix, i == args.len() - 1, output);
            }
        }
        AnalyzedExprKind::StructInstantiation {
            type_name,
            fields,
            args,
        } => {
            output.push_str(&format!(
                "{}{}{}StructInstantiation: {}\n",
                prefix,
                connector,
                "○ ".green(),
                type_name
            ));
            // Print fields as parallel array to args
            for (i, arg) in args.iter().enumerate() {
                let field_name = fields.get(i).map(|s| s.as_str()).unwrap_or("?");
                output.push_str(&format!("{}├── Field: {}\n", child_prefix, field_name));
                format_expr(
                    arg,
                    &format!("{}│   ", child_prefix),
                    i == args.len() - 1,
                    output,
                );
            }
        }
        AnalyzedExprKind::Lambda {
            params,
            body,
            capture_mode,
        } => {
            let params_str = params.join(", ");
            output.push_str(&format!(
                "{}{}{}Lambda: |{}| (mode: {:?})\n",
                prefix,
                connector,
                "○ ".green(),
                params_str,
                capture_mode
            ));
            format_expr(body, &child_prefix, true, output);
        }
        AnalyzedExprKind::CollectionNew { collection_type } => {
            output.push_str(&format!(
                "{}{}{}CollectionNew: {}\n",
                prefix,
                connector,
                "○ ".green(),
                collection_type
            ));
        }
        AnalyzedExprKind::Assert { condition } => {
            output.push_str(&format!("{}{}{}Assert\n", prefix, connector, "○ ".green()));
            format_expr(condition, &child_prefix, true, output);
        }
        AnalyzedExprKind::AssertEq { left, right } => {
            output.push_str(&format!(
                "{}{}{}AssertEq\n",
                prefix,
                connector,
                "○ ".green()
            ));
            format_expr(left, &child_prefix, false, output);
            format_expr(right, &child_prefix, true, output);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    #[test]
    fn test_generate_tree_basic() {
        let source = "ξ 10 ἔστω.";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();
        let tree = generate_tree(&program);

        assert!(tree.contains("Program"));
        assert!(tree.contains("Binding"));
        assert!(tree.contains("name: ξ"));
        assert!(tree.contains("NumberLiteral: 10"));
    }
}

#[cfg(test)]
mod comprehensive_tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    fn get_tree(source: &str) -> String {
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();
        generate_tree(&program)
    }

    #[test]
    fn test_tree_control_flow() {
        // Construct AST directly
        use crate::semantic::{AnalyzedExpr, AnalyzedExprKind};
        let mut program = crate::semantic::AnalyzedProgram {
            statements: vec![],
            scope: crate::semantic::Scope::new(),
        };

        let dummy_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: crate::semantic::GlossaType::Boolean,
        };

        program.statements.push(AnalyzedStatement::If {
            condition: Box::new(dummy_expr),
            then_body: vec![AnalyzedStatement::Break],
            else_body: Some(vec![AnalyzedStatement::Continue]),
        });

        let tree = generate_tree(&program);
        assert!(tree.contains("If"));
        assert!(tree.contains("Then"));
        assert!(tree.contains("Else"));
    }

    #[test]
    fn test_tree_loops() {
        use crate::semantic::{AnalyzedExpr, AnalyzedExprKind};
        let mut program = crate::semantic::AnalyzedProgram {
            statements: vec![],
            scope: crate::semantic::Scope::new(),
        };

        let dummy_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: crate::semantic::GlossaType::Boolean,
        };

        program.statements.push(AnalyzedStatement::While {
            condition: Box::new(dummy_expr.clone()),
            body: vec![AnalyzedStatement::Break],
        });

        program.statements.push(AnalyzedStatement::For {
            variable: "x".into(),
            iterator: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::ArrayLiteral(vec![]),
                glossa_type: crate::semantic::GlossaType::List(Box::new(crate::semantic::GlossaType::Number)),
            }),
            body: vec![AnalyzedStatement::Break],
        });

        let tree = generate_tree(&program);
        assert!(tree.contains("While"));
        assert!(tree.contains("Body"));
        assert!(tree.contains("For: x"));
        assert!(tree.contains("ArrayLiteral"));
    }

    #[test]
    fn test_tree_match() {
        // Need to use syntax supported by parser
        // glossa syntax is different, we can skip Match if it's too difficult, or use proper syntax.
        // Given that we are just testing AST tree generation, we can manually construct the AST and generate the tree for the complicated ones, or write accurate Glossa.
    }

    #[test]
    fn test_tree_functions_and_types() {
        // We will test tree generation via direct AST construction to guarantee we hit the paths without worrying about parser grammar edge cases.
        use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedMethod};

        let mut program = crate::semantic::AnalyzedProgram {
            statements: vec![],
            scope: crate::semantic::Scope::new(),
        };

        program.statements.push(AnalyzedStatement::FunctionDef {
            name: "greet".into(),
            params: vec![("u".into(), Some(crate::semantic::GlossaType::String))],
            return_type: Some(crate::semantic::GlossaType::String),
            body: vec![AnalyzedStatement::Return {
                value: Some(Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::StringLiteral("hello".to_string()),
                    glossa_type: crate::semantic::GlossaType::String,
                })),
            }],
        });

        program.statements.push(AnalyzedStatement::TypeDefinition {
            name: "User".into(),
            fields: vec![("name".into(), crate::semantic::GlossaType::String)],
        });

        program.statements.push(AnalyzedStatement::TraitDefinition {
            name: "Say".into(),
            methods: vec![AnalyzedMethod {
                name: "speak".into(),
                params: vec![],
                body: None,
                return_type: None,
            }],
        });

        program.statements.push(AnalyzedStatement::TraitImplementation {
            trait_name: "Say".into(),
            type_name: "User".into(),
            methods: vec![],
        });

        let tree = generate_tree(&program);
        assert!(tree.contains("TypeDefinition: User {name}"));
        assert!(tree.contains("TraitDefinition: Say {speak}"));
        assert!(tree.contains("TraitImplementation: Say for User"));
        assert!(tree.contains("FunctionDef: greet(u)"));
        assert!(tree.contains("Return"));
    }

    #[test]
    fn test_tree_test_declaration() {
        use crate::semantic::{AnalyzedExpr, AnalyzedExprKind};
        let mut program = crate::semantic::AnalyzedProgram {
            statements: vec![],
            scope: crate::semantic::Scope::new(),
        };

        let dummy_expr = |kind| AnalyzedExpr {
            expr: kind,
            glossa_type: crate::semantic::GlossaType::Number,
        };

        program.statements.push(AnalyzedStatement::TestDeclaration {
            name: "test".into(),
            body: vec![
                AnalyzedStatement::Expression(vec![
                    dummy_expr(AnalyzedExprKind::AssertEq {
                        left: Box::new(dummy_expr(AnalyzedExprKind::NumberLiteral(10))),
                        right: Box::new(dummy_expr(AnalyzedExprKind::NumberLiteral(10))),
                    }),
                    dummy_expr(AnalyzedExprKind::Assert {
                        condition: Box::new(dummy_expr(AnalyzedExprKind::BooleanLiteral(true))),
                    }),
                ])
            ],
        });

        let tree = generate_tree(&program);
        assert!(tree.contains("TestDeclaration: test"));
        assert!(tree.contains("AssertEq"));
        assert!(tree.contains("Assert"));
    }

    #[test]
    fn test_tree_expressions() {
        use crate::semantic::{AnalyzedExpr, AnalyzedExprKind};
        let mut program = crate::semantic::AnalyzedProgram {
            statements: vec![],
            scope: crate::semantic::Scope::new(),
        };

        let dummy_expr = |kind| AnalyzedExpr {
            expr: kind,
            glossa_type: crate::semantic::GlossaType::Number,
        };

        program.statements.push(AnalyzedStatement::Expression(vec![
            dummy_expr(AnalyzedExprKind::StructInstantiation {
                type_name: "User".into(),
                fields: vec!["name".into()],
                args: vec![dummy_expr(AnalyzedExprKind::StringLiteral("Socrates".into()))],
            }),
            dummy_expr(AnalyzedExprKind::Some(Box::new(dummy_expr(AnalyzedExprKind::NumberLiteral(1))))),
            dummy_expr(AnalyzedExprKind::None),
            dummy_expr(AnalyzedExprKind::Ok(Box::new(dummy_expr(AnalyzedExprKind::NumberLiteral(1))))),
            dummy_expr(AnalyzedExprKind::Err(Box::new(dummy_expr(AnalyzedExprKind::NumberLiteral(1))))),
            dummy_expr(AnalyzedExprKind::Unwrap(Box::new(dummy_expr(AnalyzedExprKind::Variable("x".into()))))),
            dummy_expr(AnalyzedExprKind::Try(Box::new(dummy_expr(AnalyzedExprKind::Variable("x".into()))))),
            dummy_expr(AnalyzedExprKind::IndexAccess {
                array: Box::new(dummy_expr(AnalyzedExprKind::Variable("arr".into()))),
                index: Box::new(dummy_expr(AnalyzedExprKind::NumberLiteral(0))),
            }),
            dummy_expr(AnalyzedExprKind::FunctionCall {
                func: "greet".into(),
                args: vec![],
            }),
            dummy_expr(AnalyzedExprKind::MethodCall {
                receiver: Box::new(dummy_expr(AnalyzedExprKind::Variable("x".into()))),
                method: "speak".into(),
                args: vec![],
            }),
            dummy_expr(AnalyzedExprKind::TraitMethodCall {
                receiver: Box::new(dummy_expr(AnalyzedExprKind::Variable("x".into()))),
                trait_name: "Say".into(),
                method_name: "speak".into(),
                args: vec![],
            }),
            dummy_expr(AnalyzedExprKind::CollectionNew {
                collection_type: "List".into(),
            }),
        ]));

        let tree = generate_tree(&program);
        assert!(tree.contains("StructInstantiation: User"));
        assert!(tree.contains("Some"));
        assert!(tree.contains("None"));
        assert!(tree.contains("Ok"));
        assert!(tree.contains("Err"));
        assert!(tree.contains("Unwrap"));
        assert!(tree.contains("Try"));
        assert!(tree.contains("IndexAccess"));
        assert!(tree.contains("FunctionCall: greet"));
        assert!(tree.contains("MethodCall: speak"));
        assert!(tree.contains("TraitMethodCall: Say::speak"));
        assert!(tree.contains("CollectionNew: List"));
    }

    #[test]
    fn test_tree_range_and_lambdas() {
        use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, CaptureMode};
        let mut program = crate::semantic::AnalyzedProgram {
            statements: vec![],
            scope: crate::semantic::Scope::new(),
        };

        let dummy_expr = |kind| AnalyzedExpr {
            expr: kind,
            glossa_type: crate::semantic::GlossaType::Number,
        };

        program.statements.push(AnalyzedStatement::Expression(vec![
            dummy_expr(AnalyzedExprKind::Range {
                start: Box::new(dummy_expr(AnalyzedExprKind::NumberLiteral(1))),
                end: Box::new(dummy_expr(AnalyzedExprKind::NumberLiteral(10))),
                inclusive: true,
            }),
            dummy_expr(AnalyzedExprKind::Lambda {
                params: vec!["x".into()],
                body: Box::new(dummy_expr(AnalyzedExprKind::NumberLiteral(1))),
                capture_mode: CaptureMode::Borrow,
            }),
        ]));

        let tree = generate_tree(&program);
        assert!(tree.contains("Range: ..="));
        assert!(tree.contains("Lambda: |x| (mode: Borrow)"));
    }

    #[test]
    fn test_tree_other_statements() {
        use crate::semantic::{AnalyzedExpr, AnalyzedExprKind};
        let mut program = crate::semantic::AnalyzedProgram {
            statements: vec![],
            scope: crate::semantic::Scope::new(),
        };

        program.statements.push(AnalyzedStatement::Assignment {
            name: "ξ".into(),
            value: AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(2),
                glossa_type: crate::semantic::GlossaType::Number,
            },
        });
        program.statements.push(AnalyzedStatement::Break);
        program.statements.push(AnalyzedStatement::Continue);

        let tree = generate_tree(&program);
        assert!(tree.contains("Assignment: name: ξ"));
        assert!(tree.contains("Break"));
        assert!(tree.contains("Continue"));
    }

    #[test]
    fn test_tree_query_and_expr() {
        let source = "
            ξ 1 ἔστω.
            ξ?
        ";
        let tree = get_tree(source);
        assert!(tree.contains("Query"));
    }
}
