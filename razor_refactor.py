import os

# 1. Gnomon Refactoring
gnomon_path = "src/tools/gnomon.rs"
with open(gnomon_path, "r") as f:
    gnomon_code = f.read()

visitor_struct_pattern = """#[derive(Default)]
pub struct GnomonVisitor {
    /// The current nesting depth of loops during traversal.
    pub current_depth: usize,
    /// The maximum nesting depth encountered so far.
    pub max_depth: usize,
}

impl GnomonVisitor {
    /// Creates a new `GnomonVisitor` starting at depth 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Recursively visits a statement and updates loop depth metrics.
    ///
    /// Increases depth when entering `While` or `For` loops, and explores
    /// inner statements in branches (`If`, `Match`, functions).
    pub fn visit_statement(&mut self, stmt: &AnalyzedStatement) {
        match stmt {
            AnalyzedStatement::While { body, .. } => {
                self.current_depth += 1;
                if self.current_depth > self.max_depth {
                    self.max_depth = self.current_depth;
                }
                for s in body {
                    self.visit_statement(s);
                }
                self.current_depth -= 1;
            }
            AnalyzedStatement::For { body, .. } => {
                self.current_depth += 1;
                if self.current_depth > self.max_depth {
                    self.max_depth = self.current_depth;
                }
                for s in body {
                    self.visit_statement(s);
                }
                self.current_depth -= 1;
            }
            AnalyzedStatement::If {
                then_body,
                else_body,
                ..
            } => {
                for s in then_body {
                    self.visit_statement(s);
                }
                if let Some(else_stmts) = else_body {
                    for s in else_stmts {
                        self.visit_statement(s);
                    }
                }
            }
            AnalyzedStatement::Match { arms, .. } => {
                for (_, stmts) in arms {
                    for s in stmts {
                        self.visit_statement(s);
                    }
                }
            }
            AnalyzedStatement::FunctionDef { body, .. } => {
                for s in body {
                    self.visit_statement(s);
                }
            }
            AnalyzedStatement::TestDeclaration { body, .. } => {
                for s in body {
                    self.visit_statement(s);
                }
            }
            _ => {}
        }
    }
}"""

flat_fn = """/// Recursively calculates the maximum nesting depth of loops in a statement.
pub fn calculate_max_depth(stmt: &AnalyzedStatement) -> usize {
    match stmt {
        AnalyzedStatement::While { body, .. } | AnalyzedStatement::For { body, .. } => {
            1 + body.iter().map(calculate_max_depth).max().unwrap_or(0)
        }
        AnalyzedStatement::If {
            then_body,
            else_body,
            ..
        } => {
            let then_max = then_body.iter().map(calculate_max_depth).max().unwrap_or(0);
            let else_max = else_body
                .as_ref()
                .map(|b| b.iter().map(calculate_max_depth).max().unwrap_or(0))
                .unwrap_or(0);
            then_max.max(else_max)
        }
        AnalyzedStatement::Match { arms, .. } => arms
            .iter()
            .flat_map(|(_, stmts)| stmts.iter().map(calculate_max_depth))
            .max()
            .unwrap_or(0),
        AnalyzedStatement::FunctionDef { body, .. }
        | AnalyzedStatement::TestDeclaration { body, .. } => {
            body.iter().map(calculate_max_depth).max().unwrap_or(0)
        }
        _ => 0,
    }
}"""

gnomon_code = gnomon_code.replace(visitor_struct_pattern, flat_fn)

gnomon_code = gnomon_code.replace("""    let mut visitor = GnomonVisitor::new();
    for stmt in &program.statements {
        visitor.visit_statement(stmt);
    }""", """    let max_depth = program
        .statements
        .iter()
        .map(calculate_max_depth)
        .max()
        .unwrap_or(0);""")

gnomon_code = gnomon_code.replace("visitor.max_depth", "max_depth")

test_pattern = """    #[test]
    fn test_gnomon_while_loop() {
        let mut visitor = GnomonVisitor::new();
        let stmt = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![],
        };
        visitor.visit_statement(&stmt);
        assert_eq!(visitor.max_depth, 1);
    }

    #[test]
    fn test_gnomon_for_loop() {
        let mut visitor = GnomonVisitor::new();
        let stmt = AnalyzedStatement::For {
            variable: SmolStr::new("x"),
            iterator: dummy_expr(),
            body: vec![],
        };
        visitor.visit_statement(&stmt);
        assert_eq!(visitor.max_depth, 1);
    }

    #[test]
    fn test_gnomon_nested_loops() {
        let mut visitor = GnomonVisitor::new();
        let inner_loop = AnalyzedStatement::For {
            variable: SmolStr::new("y"),
            iterator: dummy_expr(),
            body: vec![],
        };
        let outer_loop = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![inner_loop],
        };
        visitor.visit_statement(&outer_loop);
        assert_eq!(visitor.max_depth, 2);
    }"""

test_replace = """    #[test]
    fn test_gnomon_while_loop() {
        let stmt = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![],
        };
        assert_eq!(calculate_max_depth(&stmt), 1);
    }

    #[test]
    fn test_gnomon_for_loop() {
        let stmt = AnalyzedStatement::For {
            variable: SmolStr::new("x"),
            iterator: dummy_expr(),
            body: vec![],
        };
        assert_eq!(calculate_max_depth(&stmt), 1);
    }

    #[test]
    fn test_gnomon_nested_loops() {
        let inner_loop = AnalyzedStatement::For {
            variable: SmolStr::new("y"),
            iterator: dummy_expr(),
            body: vec![],
        };
        let outer_loop = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![inner_loop],
        };
        assert_eq!(calculate_max_depth(&outer_loop), 2);
    }"""

gnomon_code = gnomon_code.replace(test_pattern, test_replace)

with open(gnomon_path, "w") as f:
    f.write(gnomon_code)


# 2. Codegen Refactoring - Removing TraitMethodParts
codegen_path = "src/codegen.rs"
with open(codegen_path, "r") as f:
    codegen_code = f.read()

trait_parts_pattern = """/// Holds the constituent parts of a trait method for generation.
pub(crate) struct TraitMethodParts {
    /// The generated method signature (e.g., `fn foo(&self, x: i32)`).
    pub signature: String,
    /// The generated default body if any, or None for abstract methods.
    pub body: Option<String>,
}"""

codegen_code = codegen_code.replace(trait_parts_pattern, "")

codegen_code = codegen_code.replace(
    "pub(crate) fn generate_trait_method(&self, method: &AnalyzedMethod) -> TraitMethodParts {",
    "pub(crate) fn generate_trait_method(&self, method: &AnalyzedMethod) -> (String, Option<String>) {"
)

codegen_code = codegen_code.replace(
    "        TraitMethodParts { signature, body }\n    }",
    "        (signature, body)\n    }"
)

codegen_code = codegen_code.replace(
    "            let TraitMethodParts { signature, body } = self.generate_trait_method(method);",
    "            let (signature, body) = self.generate_trait_method(method);"
)

with open(codegen_path, "w") as f:
    f.write(codegen_code)


# 3. Flatten Errors - Merge assembly.rs into mod.rs
mod_path = "src/errors/mod.rs"
assembly_path = "src/errors/assembly.rs"

with open(assembly_path, "r") as f:
    assembly_code = f.read()

assembly_code = assembly_code.replace("use super::GlossaError;", "")

with open(mod_path, "r") as f:
    mod_code = f.read()

mod_code = mod_code.replace("pub mod assembly;", "")

# find the end of mod.rs and append assembly code
with open(mod_path, "w") as f:
    f.write(mod_code + "\n\n" + assembly_code)

os.remove(assembly_path)

print("Refactoring complete.")
