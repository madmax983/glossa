use crate::semantic::{AnalyzedMethod, AnalyzedProgram, AnalyzedStatement};
use miette::Result;
use std::path::Path;

/// Maximum nesting depth before Hubris is triggered
const MAX_NESTING_DEPTH: usize = 3;
/// Maximum statements in a function before Labyrinth is triggered
const MAX_FUNCTION_STATEMENTS: usize = 15;
/// Maximum parameters before Polymath is triggered
const MAX_FUNCTION_PARAMS: usize = 4;
/// Maximum fields in a struct before Monolith is triggered
const MAX_STRUCT_FIELDS: usize = 6;

#[derive(Debug, PartialEq)]
pub enum SmellKind {
    /// Ὕβρις (Hubris) - Reaching too close to the sun (deeply nested control flow)
    Hubris,
    /// Λαβύρινθος (Labyrinth) - Getting lost in the maze (long functions)
    Labyrinth,
    /// Πολυμαθής (Polymath) - Knowing too much (too many parameters)
    Polymath,
    /// Μονόλιθος (Monolith) - A heavy burden (too many struct fields)
    Monolith,
}

#[derive(Debug)]
pub struct Smell {
    pub kind: SmellKind,
    pub location: String,
    pub description: String,
    pub maxim: &'static str,
}

impl Smell {
    pub fn new(kind: SmellKind, location: String, description: String) -> Self {
        let maxim = match kind {
            SmellKind::Hubris => "You reach too close to the sun. Simplify thy branches.",
            SmellKind::Labyrinth => "Even the Minotaur would get lost in this function.",
            SmellKind::Polymath => "A mind divided cannot focus. Reduce thy parameters.",
            SmellKind::Monolith => "A heavy burden breaks the strongest back.",
        };
        Self {
            kind,
            location,
            description,
            maxim,
        }
    }
}

pub fn analyze_philosophy(program: &AnalyzedProgram) -> Vec<Smell> {
    let mut smells = Vec::new();

    for stmt in &program.statements {
        analyze_statement(stmt, 0, "Global".to_string(), &mut smells);
    }

    smells
}

fn analyze_statement(
    stmt: &AnalyzedStatement,
    depth: usize,
    context: String,
    smells: &mut Vec<Smell>,
) {
    if depth > MAX_NESTING_DEPTH {
        smells.push(Smell::new(
            SmellKind::Hubris,
            context.clone(),
            format!("Control flow nested to depth {}", depth),
        ));
        // We only report the first violation per branch to avoid spamming
        return;
    }

    match stmt {
        AnalyzedStatement::If {
            condition: _,
            then_body,
            else_body,
        } => {
            for s in then_body {
                analyze_statement(s, depth + 1, context.clone(), smells);
            }
            if let Some(e) = else_body {
                for s in e {
                    analyze_statement(s, depth + 1, context.clone(), smells);
                }
            }
        }
        AnalyzedStatement::While { condition: _, body } => {
            for s in body {
                analyze_statement(s, depth + 1, context.clone(), smells);
            }
        }
        AnalyzedStatement::For { variable: _, iterator: _, body } => {
            for s in body {
                analyze_statement(s, depth + 1, context.clone(), smells);
            }
        }
        AnalyzedStatement::Match { scrutinee: _, arms } => {
            for (_, body) in arms {
                for s in body {
                    analyze_statement(s, depth + 1, context.clone(), smells);
                }
            }
        }
        AnalyzedStatement::FunctionDef {
            name, params, body, return_type: _
        } => {
            let func_name = name.to_string();
            if params.len() > MAX_FUNCTION_PARAMS {
                smells.push(Smell::new(
                    SmellKind::Polymath,
                    func_name.clone(),
                    format!("Function has {} parameters", params.len()),
                ));
            }
            if body.len() > MAX_FUNCTION_STATEMENTS {
                smells.push(Smell::new(
                    SmellKind::Labyrinth,
                    func_name.clone(),
                    format!("Function contains {} statements", body.len()),
                ));
            }
            for s in body {
                analyze_statement(s, 1, func_name.clone(), smells);
            }
        }
        AnalyzedStatement::TypeDefinition { name, fields } => {
            if fields.len() > MAX_STRUCT_FIELDS {
                smells.push(Smell::new(
                    SmellKind::Monolith,
                    name.to_string(),
                    format!("Struct has {} fields", fields.len()),
                ));
            }
        }
        AnalyzedStatement::TraitDefinition { methods, .. } => {
            for method in methods {
                analyze_method(method, smells);
            }
        }
        AnalyzedStatement::TraitImplementation {
            trait_name: _, type_name, methods
        } => {
            let impl_context = format!("Impl {}", type_name);
            for method in methods {
                analyze_method_in_impl(method, &impl_context, smells);
            }
        }
        AnalyzedStatement::TestDeclaration { name, body } => {
            let test_context = format!("Test '{}'", name);
            if body.len() > MAX_FUNCTION_STATEMENTS {
                smells.push(Smell::new(
                    SmellKind::Labyrinth,
                    test_context.clone(),
                    format!("Test contains {} statements", body.len()),
                ));
            }
            for s in body {
                analyze_statement(s, 1, test_context.clone(), smells);
            }
        }
        _ => {}
    }
}

fn analyze_method(method: &AnalyzedMethod, smells: &mut Vec<Smell>) {
    let func_name = method.name.to_string();
    // Exclude 'self' conceptually by just counting all params.
    if method.params.len() > MAX_FUNCTION_PARAMS {
        smells.push(Smell::new(
            SmellKind::Polymath,
            func_name.clone(),
            format!("Method has {} parameters", method.params.len()),
        ));
    }
    if let Some(body) = &method.body {
        if body.len() > MAX_FUNCTION_STATEMENTS {
            smells.push(Smell::new(
                SmellKind::Labyrinth,
                func_name.clone(),
                format!("Method contains {} statements", body.len()),
            ));
        }
        for s in body {
            analyze_statement(s, 1, func_name.clone(), smells);
        }
    }
}

fn analyze_method_in_impl(method: &AnalyzedMethod, context: &str, smells: &mut Vec<Smell>) {
    let func_name = format!("{}::{}", context, method.name);
    if method.params.len() > MAX_FUNCTION_PARAMS {
        smells.push(Smell::new(
            SmellKind::Polymath,
            func_name.clone(),
            format!("Method has {} parameters", method.params.len()),
        ));
    }
    if let Some(body) = &method.body {
        if body.len() > MAX_FUNCTION_STATEMENTS {
            smells.push(Smell::new(
                SmellKind::Labyrinth,
                func_name.clone(),
                format!("Method contains {} statements", body.len()),
            ));
        }
        for s in body {
            analyze_statement(s, 1, func_name.clone(), smells);
        }
    }
}

pub fn run_philosopher(file_path: &Path) -> Result<()> {
    use crate::parser::parse;
    use crate::semantic::analyze_program;
    use crate::tools::runner::load_source;

    let source = load_source(file_path)?;
    let ast = parse(&source)?;
    let program = analyze_program(&ast)?;

    let smells = analyze_philosophy(&program);

    if smells.is_empty() {
        println!("The Philosopher speaks: \"Thy code is pure. No Hubris found.\"");
    } else {
        println!("The Philosopher speaks:\n");
        for smell in smells {
            let name = match smell.kind {
                SmellKind::Hubris => "Ὕβρις (Hubris)",
                SmellKind::Labyrinth => "Λαβύρινθος (Labyrinth)",
                SmellKind::Polymath => "Πολυμαθής (Polymath)",
                SmellKind::Monolith => "Μονόλιθος (Monolith)",
            };
            println!("  [{}] in `{}`", name, smell.location);
            println!("    Reason: {}", smell.description);
            println!("    Maxim:  \"{}\"\n", smell.maxim);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    #[test]
    fn test_philosopher_monolith() {
        let source = "
            εἶδος Τέρας ὁρίζειν {
                α ἀριθμοῦ.
                β ἀριθμοῦ.
                γ ἀριθμοῦ.
                δ ἀριθμοῦ.
                ε ἀριθμοῦ.
                ζ ἀριθμοῦ.
                η ἀριθμοῦ.
            }.
        ";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();
        let smells = analyze_philosophy(&program);

        assert_eq!(smells.len(), 1);
        assert_eq!(smells[0].kind, SmellKind::Monolith);
    }

    #[test]
    fn test_philosopher_polymath() {
        // Here we test polymath using a long parameter list function definition
        // Let's create a dummy AnalyzedProgram directly since our parser might have limits or grammar restrictions on 5 parameters
        use crate::semantic::GlossaType;
        use crate::semantic::Scope;
        use smol_str::SmolStr;

        let scope = Scope::new();

        let stmt = AnalyzedStatement::FunctionDef {
            name: SmolStr::new("too_many_args"),
            params: vec![
                (SmolStr::new("a"), Some(GlossaType::Number)),
                (SmolStr::new("b"), Some(GlossaType::Number)),
                (SmolStr::new("c"), Some(GlossaType::Number)),
                (SmolStr::new("d"), Some(GlossaType::Number)),
                (SmolStr::new("e"), Some(GlossaType::Number)),
            ],
            body: vec![],
            return_type: None,
        };

        let program = AnalyzedProgram {
            statements: vec![stmt],
            scope,
        };

        let smells = analyze_philosophy(&program);
        assert_eq!(smells.len(), 1);
        assert_eq!(smells[0].kind, SmellKind::Polymath);
    }

    #[test]
    fn test_philosopher_labyrinth() {
        use crate::semantic::Scope;
        use smol_str::SmolStr;

        let scope = Scope::new();
        let mut body = vec![];
        for _ in 0..16 {
            body.push(AnalyzedStatement::Expression(vec![]));
        }

        let stmt = AnalyzedStatement::FunctionDef {
            name: SmolStr::new("long_function"),
            params: vec![],
            body,
            return_type: None,
        };

        let program = AnalyzedProgram {
            statements: vec![stmt],
            scope,
        };

        let smells = analyze_philosophy(&program);
        assert_eq!(smells.len(), 1);
        assert_eq!(smells[0].kind, SmellKind::Labyrinth);
    }

    #[test]
    fn test_philosopher_hubris() {
        use crate::semantic::GlossaType;
        use crate::semantic::Scope;
        use crate::semantic::{AnalyzedExpr, AnalyzedExprKind};
        use smol_str::SmolStr;

        let scope = Scope::new();

        let cond = Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: GlossaType::Boolean,
        });

        // nesting 4 levels deep
        let inner3 = AnalyzedStatement::If {
            condition: cond.clone(),
            then_body: vec![],
            else_body: None,
        };

        let inner2 = AnalyzedStatement::If {
            condition: cond.clone(),
            then_body: vec![inner3],
            else_body: None,
        };

        let inner1 = AnalyzedStatement::If {
            condition: cond.clone(),
            then_body: vec![inner2],
            else_body: None,
        };

        let outer = AnalyzedStatement::If {
            condition: cond.clone(),
            then_body: vec![inner1],
            else_body: None,
        };

        let stmt = AnalyzedStatement::FunctionDef {
            name: SmolStr::new("nested_function"),
            params: vec![],
            body: vec![outer],
            return_type: None,
        };

        let program = AnalyzedProgram {
            statements: vec![stmt],
            scope,
        };

        let smells = analyze_philosophy(&program);
        assert_eq!(smells.len(), 1);
        assert_eq!(smells[0].kind, SmellKind::Hubris);
    }
}
