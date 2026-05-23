import re
import subprocess

subprocess.run(['git', 'checkout', 'src/tools/auditor.rs'])

with open('src/tools/auditor.rs', 'r') as f:
    content = f.read()

new_fn = """pub fn audit_statement(
    stmt: &AnalyzedStatement,
    usage_count: &mut FxHashMap<SmolStr, usize>,
    mutation_count: &mut FxHashMap<SmolStr, usize>,
    mutable_vars: &mut FxHashSet<SmolStr>,
) {
    match stmt {
        AnalyzedStatement::Binding { name, value, mutable } => {
            usage_count.insert(name.clone(), 0);
            mutation_count.insert(name.clone(), 0);
            if *mutable {
                mutable_vars.insert(name.clone());
            }
            audit_expr(value, usage_count, mutation_count, mutable_vars);
        }
        AnalyzedStatement::Assignment { name, value } => {
            if let Some(count) = mutation_count.get_mut(name) {
                *count += 1;
            }
            if let Some(count) = usage_count.get_mut(name) {
                *count += 1;
            }
            audit_expr(value, usage_count, mutation_count, mutable_vars);
        }
        AnalyzedStatement::Print(exprs) => {
            for expr in exprs {
                audit_expr(expr, usage_count, mutation_count, mutable_vars);
            }
        }
        AnalyzedStatement::Expression(exprs) => {
            for expr in exprs {
                audit_expr(expr, usage_count, mutation_count, mutable_vars);
            }
        }
        AnalyzedStatement::Query(exprs) => {
            for expr in exprs {
                audit_expr(expr, usage_count, mutation_count, mutable_vars);
            }
        }
        AnalyzedStatement::If { condition, then_body, else_body } => {
            audit_expr(condition, usage_count, mutation_count, mutable_vars);
            for s in then_body {
                audit_statement(s, usage_count, mutation_count, mutable_vars);
            }
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    audit_statement(s, usage_count, mutation_count, mutable_vars);
                }
            }
        }
        AnalyzedStatement::While { condition, body } => {
            audit_expr(condition, usage_count, mutation_count, mutable_vars);
            for s in body {
                audit_statement(s, usage_count, mutation_count, mutable_vars);
            }
        }
        AnalyzedStatement::For { variable, iterator, body } => {
            usage_count.insert(variable.clone(), 0);
            audit_expr(iterator, usage_count, mutation_count, mutable_vars);
            for s in body {
                audit_statement(s, usage_count, mutation_count, mutable_vars);
            }
        }
        AnalyzedStatement::Match { scrutinee, arms } => {
            audit_expr(scrutinee, usage_count, mutation_count, mutable_vars);
            for (expr, stmts) in arms {
                audit_expr(expr, usage_count, mutation_count, mutable_vars);
                for s in stmts {
                    audit_statement(s, usage_count, mutation_count, mutable_vars);
                }
            }
        }
        AnalyzedStatement::FunctionDef { params, body, .. } => {
            for (param_name, _) in params {
                usage_count.insert(param_name.clone(), 0);
            }
            for s in body {
                audit_statement(s, usage_count, mutation_count, mutable_vars);
            }
        }
        AnalyzedStatement::Return { value } => {
            if let Some(v) = value {
                audit_expr(v, usage_count, mutation_count, mutable_vars);
            }
        }
        AnalyzedStatement::TestDeclaration { body, .. } => {
            for s in body {
                audit_statement(s, usage_count, mutation_count, mutable_vars);
            }
        }
        AnalyzedStatement::Break
        | AnalyzedStatement::Continue
        | AnalyzedStatement::TypeDefinition { .. }
        | AnalyzedStatement::TraitDefinition { .. }
        | AnalyzedStatement::TraitImplementation { .. } => {}
    }
}

pub fn audit_expr(
    expr: &AnalyzedExpr,
    usage_count: &mut FxHashMap<SmolStr, usize>,
    mutation_count: &mut FxHashMap<SmolStr, usize>,
    mutable_vars: &mut FxHashSet<SmolStr>,
) {
    match &expr.expr {
        AnalyzedExprKind::Variable(name) => {
            if let Some(count) = usage_count.get_mut(name) {
                *count += 1;
            }
        }
        AnalyzedExprKind::BinOp { left, right, .. } => {
            audit_expr(left, usage_count, mutation_count, mutable_vars);
            audit_expr(right, usage_count, mutation_count, mutable_vars);
        }
        AnalyzedExprKind::UnaryOp { operand, .. } => {
            audit_expr(operand, usage_count, mutation_count, mutable_vars)
        }
        AnalyzedExprKind::StructInstantiation { args, .. } => {
            for arg in args {
                audit_expr(arg, usage_count, mutation_count, mutable_vars);
            }
        }
        AnalyzedExprKind::PropertyAccess { owner, .. } => {
            audit_expr(owner, usage_count, mutation_count, mutable_vars)
        }
        AnalyzedExprKind::MethodCall { receiver, args, .. } => {
            audit_expr(receiver, usage_count, mutation_count, mutable_vars);
            for arg in args {
                audit_expr(arg, usage_count, mutation_count, mutable_vars);
            }
        }
        AnalyzedExprKind::FunctionCall { args, .. } | AnalyzedExprKind::VerbCall { args, .. } => {
            for arg in args {
                audit_expr(arg, usage_count, mutation_count, mutable_vars);
            }
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => {
            for arg in exprs {
                audit_expr(arg, usage_count, mutation_count, mutable_vars);
            }
        }
        AnalyzedExprKind::IndexAccess { array, index } => {
            audit_expr(array, usage_count, mutation_count, mutable_vars);
            audit_expr(index, usage_count, mutation_count, mutable_vars);
        }
        AnalyzedExprKind::Lambda { body, .. } => {
            audit_expr(body, usage_count, mutation_count, mutable_vars)
        }
        AnalyzedExprKind::Some(inner)
        | AnalyzedExprKind::Ok(inner)
        | AnalyzedExprKind::Err(inner)
        | AnalyzedExprKind::Unwrap(inner)
        | AnalyzedExprKind::Try(inner) => audit_expr(inner, usage_count, mutation_count, mutable_vars),
        AnalyzedExprKind::Assert { condition } => {
            audit_expr(condition, usage_count, mutation_count, mutable_vars)
        }
        AnalyzedExprKind::AssertEq { left, right } => {
            audit_expr(left, usage_count, mutation_count, mutable_vars);
            audit_expr(right, usage_count, mutation_count, mutable_vars);
        }
        AnalyzedExprKind::Range { start, end, .. } => {
            audit_expr(start, usage_count, mutation_count, mutable_vars);
            audit_expr(end, usage_count, mutation_count, mutable_vars);
        }
        AnalyzedExprKind::NumberLiteral(_)
        | AnalyzedExprKind::StringLiteral(_)
        | AnalyzedExprKind::BooleanLiteral(_)
        | AnalyzedExprKind::None
        | AnalyzedExprKind::CollectionNew { .. } => {}
    }
}
"""

start_str = "struct AuditorVisitor {"
end_str = "        }\n    }\n}\n"

start_idx = content.find(start_str)
end_idx = content.find(end_str, start_idx) + len(end_str)

if start_idx != -1 and end_idx != -1:
    content = content[:start_idx] + new_fn + content[end_idx:]

# Update run_auditor
content = content.replace(
"""    let mut visitor = AuditorVisitor::new();
    for stmt in &program.statements {
        visitor.visit_statement(stmt);
    }""",
"""    let mut usage_count = FxHashMap::default();
    let mut mutation_count = FxHashMap::default();
    let mut mutable_vars = FxHashSet::default();
    for stmt in &program.statements {
        audit_statement(stmt, &mut usage_count, &mut mutation_count, &mut mutable_vars);
    }"""
)

content = content.replace("visitor.usage_count", "usage_count")
content = content.replace("visitor.mutation_count", "mutation_count")
content = content.replace("visitor.mutable_vars", "mutable_vars")

# Fix tests
content = content.replace("let mut visitor = AuditorVisitor::new();", "let mut usage_count = FxHashMap::default();\n        let mut mutation_count = FxHashMap::default();\n        let mut mutable_vars = FxHashSet::default();")
content = content.replace("visitor.visit_statement(", "audit_statement(")
content = content.replace("visitor.visit_expr(&expr);", "audit_expr(&expr, &mut usage_count, &mut mutation_count, &mut mutable_vars);")

content = re.sub(
    r'audit_statement\((.*?)\);',
    r'audit_statement(\1, &mut usage_count, &mut mutation_count, &mut mutable_vars);',
    content
)


with open('src/tools/auditor.rs', 'w') as f:
    f.write(content)
