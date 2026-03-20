//! The Tree Tool ("The Arborist" / τὸ Δένδρον)
//!
//! This module implements a CLI tool to visualize the Abstract Syntax Tree (AST)
//! of a ΓΛΩΣΣΑ program.

use crate::ast::{Clause, Expr, Statement};
use crate::parser::parse;
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::Result;
use std::io::Write;
use std::path::Path;

/// Run the Tree tool on a file
pub fn run_tree(input: &Path) -> Result<()> {
    let status = Status::start_with_symbol("Δένδρον (AST Tree)", "🌳");

    let source = load_source(input)?;

    let mut buffer = Vec::new();
    run_tree_inner(&source, &mut buffer)?;

    let output = String::from_utf8_lossy(&buffer);

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   T R E E".bold().cyan());
    println!("   {}", "Abstract Syntax Tree".italic().dim());
    println!();
    println!("{}", output);

    Ok(())
}

/// Inner function to parse and write the tree to a buffer (useful for testing)
pub fn run_tree_inner(source: &str, output: &mut impl Write) -> Result<()> {
    let ast = parse(source).map_err(|e| miette::miette!("Parse error: {}", e))?;

    writeln!(output, "{}", "Program".bold().green()).unwrap();

    for (i, stmt) in ast.statements.iter().enumerate() {
        let is_last = i == ast.statements.len() - 1;
        print_statement(stmt, output, "", is_last);
    }

    Ok(())
}

fn print_statement(stmt: &Statement, output: &mut impl Write, prefix: &str, is_last: bool) {
    let marker = if is_last { "└── " } else { "├── " };
    let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

    match stmt {
        Statement::Regular {
            clauses,
            is_query,
            is_propagate,
        } => {
            writeln!(
                output,
                "{}{}{}",
                prefix,
                marker,
                format!(
                    "Statement::Regular (query={}, propagate={})",
                    is_query, is_propagate
                )
                .yellow()
            )
            .unwrap();

            for (i, clause) in clauses.iter().enumerate() {
                let clause_last = i == clauses.len() - 1;
                print_clause(clause, output, &child_prefix, clause_last);
            }
        }
        Statement::TypeDefinition(td) => {
            writeln!(
                output,
                "{}{}{}",
                prefix,
                marker,
                "Statement::TypeDefinition".yellow()
            )
            .unwrap();
            writeln!(
                output,
                "{}└── name: {}",
                child_prefix,
                td.name.original.as_str().cyan()
            )
            .unwrap();
            // In a full implementation, we would print fields too
        }
        Statement::TraitDefinition(td) => {
            writeln!(
                output,
                "{}{}{}",
                prefix,
                marker,
                "Statement::TraitDefinition".yellow()
            )
            .unwrap();
            writeln!(
                output,
                "{}└── name: {}",
                child_prefix,
                td.name.original.as_str().cyan()
            )
            .unwrap();
        }
        Statement::TraitImpl(ti) => {
            writeln!(
                output,
                "{}{}{}",
                prefix,
                marker,
                "Statement::TraitImpl".yellow()
            )
            .unwrap();
            writeln!(
                output,
                "{}├── type: {}",
                child_prefix,
                ti.type_name.original.as_str().cyan()
            )
            .unwrap();
            writeln!(
                output,
                "{}└── trait: {}",
                child_prefix,
                ti.trait_name.original.as_str().cyan()
            )
            .unwrap();
        }
        Statement::TestDeclaration(td) => {
            writeln!(
                output,
                "{}{}{}",
                prefix,
                marker,
                "Statement::TestDeclaration".yellow()
            )
            .unwrap();
            writeln!(
                output,
                "{}├── name: {}",
                child_prefix,
                td.name.clone().cyan()
            )
            .unwrap();

            for (i, s) in td.body.iter().enumerate() {
                let s_last = i == td.body.len() - 1;
                print_statement(s, output, &child_prefix, s_last);
            }
        }
    }
}

fn print_clause(clause: &Clause, output: &mut impl Write, prefix: &str, is_last: bool) {
    let marker = if is_last { "└── " } else { "├── " };
    let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

    writeln!(output, "{}{}{}", prefix, marker, "Clause".blue()).unwrap();

    for (i, expr) in clause.expressions.iter().enumerate() {
        let expr_last = i == clause.expressions.len() - 1;
        print_expr(expr, output, &child_prefix, expr_last);
    }
}

fn print_expr(expr: &Expr, output: &mut impl Write, prefix: &str, is_last: bool) {
    let marker = if is_last { "└── " } else { "├── " };
    let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

    match expr {
        Expr::StringLiteral(s) => writeln!(
            output,
            "{}{}{}({})",
            prefix,
            marker,
            "StringLiteral".cyan(),
            s
        )
        .unwrap(),
        Expr::NumberLiteral(n) => writeln!(
            output,
            "{}{}{}({})",
            prefix,
            marker,
            "NumberLiteral".cyan(),
            n
        )
        .unwrap(),
        Expr::BooleanLiteral(b) => writeln!(
            output,
            "{}{}{}({})",
            prefix,
            marker,
            "BooleanLiteral".cyan(),
            b
        )
        .unwrap(),
        Expr::ArrayLiteral(exprs) => {
            writeln!(output, "{}{}{}", prefix, marker, "ArrayLiteral".cyan()).unwrap();
            for (i, e) in exprs.iter().enumerate() {
                let e_last = i == exprs.len() - 1;
                print_expr(e, output, &child_prefix, e_last);
            }
        }
        Expr::IndexAccess { array, index } => {
            writeln!(output, "{}{}{}", prefix, marker, "IndexAccess".cyan()).unwrap();
            print_expr(array, output, &child_prefix, false);
            print_expr(index, output, &child_prefix, true);
        }
        Expr::Word(w) => writeln!(
            output,
            "{}{}{}({})",
            prefix,
            marker,
            "Word".cyan(),
            w.original
        )
        .unwrap(),
        Expr::Phrase(exprs) => {
            writeln!(output, "{}{}{}", prefix, marker, "Phrase".cyan()).unwrap();
            for (i, e) in exprs.iter().enumerate() {
                let e_last = i == exprs.len() - 1;
                print_expr(e, output, &child_prefix, e_last);
            }
        }
        Expr::PropertyAccess { owner, property } => {
            writeln!(output, "{}{}{}", prefix, marker, "PropertyAccess".cyan()).unwrap();
            print_expr(owner, output, &child_prefix, false);
            print_expr(property, output, &child_prefix, true);
        }
        Expr::Call { verb, arguments } => {
            writeln!(
                output,
                "{}{}{}({})",
                prefix,
                marker,
                "Call".cyan(),
                verb.original
            )
            .unwrap();
            for (i, e) in arguments.iter().enumerate() {
                let e_last = i == arguments.len() - 1;
                print_expr(e, output, &child_prefix, e_last);
            }
        }
        Expr::Binding { name, value } => {
            writeln!(
                output,
                "{}{}{}({})",
                prefix,
                marker,
                "Binding".magenta(),
                name.original
            )
            .unwrap();
            print_expr(value, output, &child_prefix, true);
        }
        Expr::BinOp { left, op, right } => {
            writeln!(output, "{}{}{:?} {}", prefix, marker, op, "(BinOp)".cyan()).unwrap();
            print_expr(left, output, &child_prefix, false);
            print_expr(right, output, &child_prefix, true);
        }
        Expr::UnaryOp { op, operand } => {
            writeln!(
                output,
                "{}{}{:?} {}",
                prefix,
                marker,
                op,
                "(UnaryOp)".cyan()
            )
            .unwrap();
            print_expr(operand, output, &child_prefix, true);
        }
        Expr::Block(stmts) => {
            writeln!(output, "{}{}{}", prefix, marker, "Block".cyan()).unwrap();
            for (i, s) in stmts.iter().enumerate() {
                let s_last = i == stmts.len() - 1;
                print_statement(s, output, &child_prefix, s_last);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_binding() {
        let source = "ξ πέντε ἔστω.";
        let mut buffer = Vec::new();
        run_tree_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Program"));
        assert!(output.contains("Statement::Regular"));
        assert!(output.contains("Clause"));
        // At the AST stage, standard sentences are parsed as a Phrase of Words
        // before semantic analysis turns them into Bindings.
        assert!(output.contains("Phrase"));
        assert!(output.contains("Word"));
        assert!(output.contains("ξ"));
        assert!(output.contains("πέντε"));
    }

    #[test]
    fn test_tree_type_def() {
        let source = "εἶδος Χρήστης ὁρίζειν { x ἀριθμοῦ. }.";
        let mut buffer = Vec::new();
        run_tree_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Statement::TypeDefinition"));
        assert!(output.contains("Χρήστης"));
    }

    #[test]
    fn test_tree_trait_def() {
        let source = "χαρακτήρ Ὄχημα ὁρίζειν { }.";
        let mut buffer = Vec::new();
        run_tree_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Statement::TraitDefinition"));
        assert!(output.contains("Ὄχημα"));
    }

    #[test]
    fn test_tree_trait_impl() {
        let source = "εἶδος Αὐτοκίνητον τῷ Ὄχημα ἐμπίπτειν { }.";
        let mut buffer = Vec::new();
        run_tree_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Statement::TraitImpl"));
        assert!(output.contains("Αὐτοκίνητον"));
        assert!(output.contains("Ὄχημα"));
    }

    #[test]
    fn test_tree_test_decl() {
        let source = "δοκιμή «τεστ». x 5 ἔστω. τέλος.";
        let mut buffer = Vec::new();
        run_tree_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Statement::TestDeclaration"));
        assert!(output.contains("τεστ"));
    }

    #[test]
    fn test_tree_exprs_and_ops() {
        // Here we test multiple expressions: string, block, call, binop, array, index, property, etc.
        // using the manual generation since we just want to cover the branches in tree.rs,
        // or we can test complex phrases directly.
        use crate::ast::{BinOperator, UnaryOperator, Word};

        let mut output = Vec::new();

        // 1. Array Literal
        let arr = Expr::ArrayLiteral(vec![Expr::NumberLiteral(1)]);
        print_expr(&arr, &mut output, "", true);

        // 2. Index Access
        let idx = Expr::IndexAccess {
            array: Box::new(Expr::Word(Word::new("x"))),
            index: Box::new(Expr::NumberLiteral(0)),
        };
        print_expr(&idx, &mut output, "", true);

        // 3. Property Access
        let prop = Expr::PropertyAccess {
            owner: Box::new(Expr::Word(Word::new("obj"))),
            property: Box::new(Expr::Word(Word::new("prop"))),
        };
        print_expr(&prop, &mut output, "", true);

        // 4. Call
        let call = Expr::Call {
            verb: Word::new("λέγε"),
            arguments: vec![Expr::StringLiteral("test".into())],
        };
        print_expr(&call, &mut output, "", true);

        // 5. BinOp
        let binop = Expr::BinOp {
            left: Box::new(Expr::NumberLiteral(1)),
            op: BinOperator::Add,
            right: Box::new(Expr::NumberLiteral(2)),
        };
        print_expr(&binop, &mut output, "", true);

        // 6. UnaryOp
        let unary = Expr::UnaryOp {
            op: UnaryOperator::Not,
            operand: Box::new(Expr::BooleanLiteral(true)),
        };
        print_expr(&unary, &mut output, "", true);

        // 7. Block
        let block = Expr::Block(vec![]);
        print_expr(&block, &mut output, "", true);

        // 8. Binding (already partially covered, but just explicitly here)
        let binding = Expr::Binding {
            name: Word::new("x"),
            value: Box::new(Expr::NumberLiteral(1)),
        };
        print_expr(&binding, &mut output, "", true);

        let out_str = String::from_utf8(output).unwrap();
        assert!(out_str.contains("ArrayLiteral"));
        assert!(out_str.contains("IndexAccess"));
        assert!(out_str.contains("PropertyAccess"));
        assert!(out_str.contains("Call"));
        assert!(out_str.contains("BinOp"));
        assert!(out_str.contains("UnaryOp"));
        assert!(out_str.contains("Block"));
        assert!(out_str.contains("Binding"));
        assert!(out_str.contains("StringLiteral"));
        assert!(out_str.contains("BooleanLiteral"));
    }

    #[test]
    fn test_tree_run_tree_file_io() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("tree_test.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("«χαῖρε κόσμε» λέγε.\n".as_bytes()).unwrap();
        }

        let result = run_tree(&input_path);
        assert!(result.is_ok());
    }
}
