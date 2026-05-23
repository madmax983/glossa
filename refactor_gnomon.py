import re
import subprocess

subprocess.run(['git', 'checkout', 'src/tools/gnomon.rs'])

with open('src/tools/gnomon.rs', 'r') as f:
    content = f.read()

# Replace GnomonVisitor struct and impl
new_fn = """/// Recursively visits a statement and updates loop depth metrics.
///
/// Increases depth when entering `While` or `For` loops, and explores
/// inner statements in branches (`If`, `Match`, functions).
pub fn calculate_max_depth(stmt: &AnalyzedStatement, current_depth: usize, max_depth: &mut usize) {
    match stmt {
        AnalyzedStatement::While { body, .. } => {
            let next_depth = current_depth + 1;
            if next_depth > *max_depth {
                *max_depth = next_depth;
            }
            for s in body {
                calculate_max_depth(s, next_depth, max_depth);
            }
        }
        AnalyzedStatement::For { body, .. } => {
            let next_depth = current_depth + 1;
            if next_depth > *max_depth {
                *max_depth = next_depth;
            }
            for s in body {
                calculate_max_depth(s, next_depth, max_depth);
            }
        }
        AnalyzedStatement::If {
            then_body,
            else_body,
            ..
        } => {
            for s in then_body {
                calculate_max_depth(s, current_depth, max_depth);
            }
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    calculate_max_depth(s, current_depth, max_depth);
                }
            }
        }
        AnalyzedStatement::Match { arms, .. } => {
            for (_, stmts) in arms {
                for s in stmts {
                    calculate_max_depth(s, current_depth, max_depth);
                }
            }
        }
        AnalyzedStatement::FunctionDef { body, .. } => {
            for s in body {
                calculate_max_depth(s, current_depth, max_depth);
            }
        }
        AnalyzedStatement::TestDeclaration { body, .. } => {
            for s in body {
                calculate_max_depth(s, current_depth, max_depth);
            }
        }
        _ => {}
    }
}"""

start_str = """/// A visitor that traverses the Abstract Syntax Tree to calculate loop depth.
///
/// Just as a gnomon casts a shadow to indicate time, this visitor casts a shadow
/// over the structure of a program to estimate its execution time complexity.
/// It tracks the maximum nesting depth of `while` and `for` loops.
#[derive(Default)]
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
}
"""

content = content.replace(start_str, new_fn + "\n")

# Update run_gnomon usage
content = content.replace("    let mut visitor = GnomonVisitor::new();", "    let mut max_depth = 0;")
content = content.replace("visitor.visit_statement(stmt);", "calculate_max_depth(stmt, 0, &mut max_depth);")
content = content.replace("visitor.max_depth", "max_depth")

# Update test usages
content = content.replace("let mut visitor = GnomonVisitor::new();", "let mut max_depth = 0;")
content = content.replace("visitor.visit_statement(&stmt);", "calculate_max_depth(&stmt, 0, &mut max_depth);")
content = content.replace("visitor.visit_statement(&outer_loop);", "calculate_max_depth(&outer_loop, 0, &mut max_depth);")

# Ensure doc comment in run_gnomon matches the change
content = content.replace("using the [`GnomonVisitor`].", "using the `calculate_max_depth` function.")

with open('src/tools/gnomon.rs', 'w') as f:
    f.write(content)
