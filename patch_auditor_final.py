import re

with open("src/tools/auditor.rs", "r") as f:
    code = f.read()

# I am having trouble matching the exact string for AuditorVisitor because of whitespace or something. I'll use regex.
pattern = re.compile(r"struct AuditorVisitor \{.*?impl AuditorVisitor \{.*?\}\n\}", re.DOTALL)

replace = """
pub fn visit_if_statement(
    condition: &AnalyzedExpr,
    then_body: &[AnalyzedStatement],
    else_body: &Option<Vec<AnalyzedStatement>>,
    usage_count: &mut FxHashMap<SmolStr, usize>,
    mutation_count: &mut FxHashMap<SmolStr, usize>,
    mutable_vars: &mut FxHashSet<SmolStr>,
) {
    visit_expr(condition, usage_count);
    for s in then_body {
        visit_statement(s, usage_count, mutation_count, mutable_vars);
    }
    if let Some(else_stmts) = else_body {
        for s in else_stmts {
            visit_statement(s, usage_count, mutation_count, mutable_vars);
        }
    }
}

pub fn visit_while_loop(
    condition: &AnalyzedExpr,
    body: &[AnalyzedStatement],
    usage_count: &mut FxHashMap<SmolStr, usize>,
    mutation_count: &mut FxHashMap<SmolStr, usize>,
    mutable_vars: &mut FxHashSet<SmolStr>,
) {
    visit_expr(condition, usage_count);
    for s in body {
        visit_statement(s, usage_count, mutation_count, mutable_vars);
    }
}

pub fn visit_for_loop(
    variable: &smol_str::SmolStr,
    iterator: &AnalyzedExpr,
    body: &[AnalyzedStatement],
    usage_count: &mut FxHashMap<SmolStr, usize>,
    mutation_count: &mut FxHashMap<SmolStr, usize>,
    mutable_vars: &mut FxHashSet<SmolStr>,
) {
    usage_count.insert(variable.clone(), 0);
    visit_expr(iterator, usage_count);
    for s in body {
        visit_statement(s, usage_count, mutation_count, mutable_vars);
    }
}

pub fn visit_match_statement(
    scrutinee: &AnalyzedExpr,
    arms: &[(AnalyzedExpr, Vec<AnalyzedStatement>)],
    usage_count: &mut FxHashMap<SmolStr, usize>,
    mutation_count: &mut FxHashMap<SmolStr, usize>,
    mutable_vars: &mut FxHashSet<SmolStr>,
) {
    visit_expr(scrutinee, usage_count);
    for (expr, stmts) in arms {
        visit_expr(expr, usage_count);
        for s in stmts {
            visit_statement(s, usage_count, mutation_count, mutable_vars);
        }
    }
}

pub fn visit_function_def(
    params: &[(smol_str::SmolStr, Option<crate::semantic::GlossaType>)],
    body: &[AnalyzedStatement],
    usage_count: &mut FxHashMap<SmolStr, usize>,
    mutation_count: &mut FxHashMap<SmolStr, usize>,
    mutable_vars: &mut FxHashSet<SmolStr>,
) {
    for (param_name, _) in params {
        usage_count.insert(param_name.clone(), 0);
    }
    for s in body {
        visit_statement(s, usage_count, mutation_count, mutable_vars);
    }
}

pub(crate) fn visit_statement(
    stmt: &AnalyzedStatement,
    usage_count: &mut FxHashMap<SmolStr, usize>,
    mutation_count: &mut FxHashMap<SmolStr, usize>,
    mutable_vars: &mut FxHashSet<SmolStr>,
) {
    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            usage_count.insert(name.clone(), 0);
            mutation_count.insert(name.clone(), 0);
            if *mutable {
                mutable_vars.insert(name.clone());
            }
            visit_expr(value, usage_count);
        }
        AnalyzedStatement::Assignment { name, value } => {
            if let Some(count) = mutation_count.get_mut(name) {
                *count += 1;
            }
            if let Some(count) = usage_count.get_mut(name) {
                *count += 1;
            }
            visit_expr(value, usage_count);
        }
        AnalyzedStatement::Print(exprs) => {
            visit_exprs(exprs, usage_count);
        }
        AnalyzedStatement::Expression(exprs) => {
            visit_exprs(exprs, usage_count);
        }
        AnalyzedStatement::Query(exprs) => {
            visit_exprs(exprs, usage_count);
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            visit_if_statement(
                condition,
                then_body,
                else_body,
                usage_count,
                mutation_count,
                mutable_vars,
            );
        }
        AnalyzedStatement::While { condition, body } => {
            visit_while_loop(condition, body, usage_count, mutation_count, mutable_vars);
        }
        AnalyzedStatement::For {
            variable,
            iterator,
            body,
        } => {
            visit_for_loop(
                variable,
                iterator,
                body,
                usage_count,
                mutation_count,
                mutable_vars,
            );
        }
        AnalyzedStatement::Match { scrutinee, arms } => {
            visit_match_statement(scrutinee, arms, usage_count, mutation_count, mutable_vars);
        }
        AnalyzedStatement::FunctionDef { params, body, .. } => {
            visit_function_def(params, body, usage_count, mutation_count, mutable_vars);
        }
        AnalyzedStatement::Return { value } => {
            if let Some(v) = value {
                visit_expr(v, usage_count);
            }
        }
        AnalyzedStatement::TestDeclaration { body, .. } => {
            for s in body {
                visit_statement(s, usage_count, mutation_count, mutable_vars);
            }
        }
        AnalyzedStatement::Break
        | AnalyzedStatement::Continue
        | AnalyzedStatement::TypeDefinition { .. }
        | AnalyzedStatement::TraitDefinition { .. }
        | AnalyzedStatement::TraitImplementation { .. } => {}
    }
}

pub fn visit_exprs(exprs: &[AnalyzedExpr], usage_count: &mut FxHashMap<SmolStr, usize>) {
    for expr in exprs {
        visit_expr(expr, usage_count);
    }
}

pub(crate) fn visit_expr(expr: &AnalyzedExpr, usage_count: &mut FxHashMap<SmolStr, usize>) {
    match &expr.expr {
        AnalyzedExprKind::Variable(name) => {
            if let Some(count) = usage_count.get_mut(name) {
                *count += 1;
            }
        }
        AnalyzedExprKind::BinOp { left, right, .. } => {
            visit_expr(left, usage_count);
            visit_expr(right, usage_count);
        }
        AnalyzedExprKind::UnaryOp { operand, .. } => visit_expr(operand, usage_count),
        AnalyzedExprKind::StructInstantiation { args, .. } => visit_exprs(args, usage_count),
        AnalyzedExprKind::PropertyAccess { owner, .. } => visit_expr(owner, usage_count),
        AnalyzedExprKind::MethodCall { receiver, args, .. } => {
            visit_expr(receiver, usage_count);
            visit_exprs(args, usage_count);
        }
        AnalyzedExprKind::FunctionCall { args, .. } => visit_exprs(args, usage_count),
        AnalyzedExprKind::VerbCall { args, .. } => visit_exprs(args, usage_count),
        AnalyzedExprKind::ArrayLiteral(exprs) => visit_exprs(exprs, usage_count),
        AnalyzedExprKind::IndexAccess { array, index } => {
            visit_expr(array, usage_count);
            visit_expr(index, usage_count);
        }
        AnalyzedExprKind::Lambda { body, .. } => visit_expr(body, usage_count),
        AnalyzedExprKind::Some(inner) => visit_expr(inner, usage_count),
        AnalyzedExprKind::Ok(inner) => visit_expr(inner, usage_count),
        AnalyzedExprKind::Err(inner) => visit_expr(inner, usage_count),
        AnalyzedExprKind::Unwrap(inner) => visit_expr(inner, usage_count),
        AnalyzedExprKind::Try(inner) => visit_expr(inner, usage_count),
        AnalyzedExprKind::Assert { condition } => visit_expr(condition, usage_count),
        AnalyzedExprKind::AssertEq { left, right } => {
            visit_expr(left, usage_count);
            visit_expr(right, usage_count);
        }
        AnalyzedExprKind::Range { start, end, .. } => {
            visit_expr(start, usage_count);
            visit_expr(end, usage_count);
        }
        AnalyzedExprKind::NumberLiteral(_)
        | AnalyzedExprKind::StringLiteral(_)
        | AnalyzedExprKind::BooleanLiteral(_)
        | AnalyzedExprKind::None
        | AnalyzedExprKind::CollectionNew { .. } => {}
    }
}
"""

if pattern.search(code):
    code = pattern.sub(replace.strip(), code)
else:
    print("Could not find AuditorVisitor struct definition with regex")


# Update run_auditor call site
search2 = """
    let mut visitor = AuditorVisitor::new();
    for stmt in &program.statements {
        visitor.visit_statement(stmt);
    }
"""

replace2 = """
    let mut usage_count = FxHashMap::default();
    let mut mutation_count = FxHashMap::default();
    let mut mutable_vars = FxHashSet::default();

    for stmt in &program.statements {
        visit_statement(
            stmt,
            &mut usage_count,
            &mut mutation_count,
            &mut mutable_vars,
        );
    }
"""
code = code.replace(search2, replace2)

# Update run_auditor issues loops
search3 = """
    for (var, count) in &visitor.usage_count {
        if *count == 0 {
            table.add_row(vec![
                Cell::new("⚠️ Unused Variable").fg(Color::Yellow),
                Cell::new(var),
                Cell::new("Declared but never used"),
            ]);
            issues += 1;
        }
    }

    for (var, count) in &visitor.mutation_count {
        if *count == 0 && visitor.mutable_vars.contains(var) && visitor.usage_count.get(var).unwrap_or(&0) > &0 {
            table.add_row(vec![
                Cell::new("💡 Unnecessary Mutation").fg(Color::Blue),
                Cell::new(var),
                Cell::new("Declared mutable ('μετά') but never changed"),
            ]);
            issues += 1;
        }
    }
"""

replace3 = """
    for (var, count) in &usage_count {
        if *count == 0 {
            table.add_row(vec![
                Cell::new("⚠️ Unused Variable").fg(Color::Yellow),
                Cell::new(var),
                Cell::new("Declared but never used"),
            ]);
            issues += 1;
        }
    }

    for (var, count) in &mutation_count {
        if *count == 0 && mutable_vars.contains(var) && usage_count.get(var).unwrap_or(&0) > &0 {
            table.add_row(vec![
                Cell::new("💡 Unnecessary Mutation").fg(Color::Blue),
                Cell::new(var),
                Cell::new("Declared mutable ('μετά') but never changed"),
            ]);
            issues += 1;
        }
    }
"""
code = code.replace(search3, replace3)


# Update tests statements
search5 = """
    #[test]
    fn test_auditor_visitor_coverage_statements() {
        let mut visitor = AuditorVisitor::new();

        let statements = vec![
"""
replace5 = """
    #[test]
    fn test_auditor_visitor_coverage_statements() {
        let mut usage_count = FxHashMap::default();
        let mut mutation_count = FxHashMap::default();
        let mut mutable_vars = FxHashSet::default();

        let statements = vec![
"""
code = code.replace(search5, replace5)

search6 = """
        for stmt in statements {
            visitor.visit_statement(&stmt);
        }
"""
replace6 = """
        for stmt in statements {
            visit_statement(
                &stmt,
                &mut usage_count,
                &mut mutation_count,
                &mut mutable_vars,
            );
        }
"""
code = code.replace(search6, replace6)


# Update tests exprs
search7 = """
    #[test]
    fn test_auditor_visitor_coverage_expressions() {
        let mut visitor = AuditorVisitor::new();

        let exprs = vec![
"""
replace7 = """
    #[test]
    fn test_auditor_visitor_coverage_expressions() {
        let mut usage_count = FxHashMap::default();

        let exprs = vec![
"""
code = code.replace(search7, replace7)

search8 = """
        for kind in exprs {
            let expr = AnalyzedExpr {
                expr: kind,
                glossa_type: crate::semantic::GlossaType::Boolean,
            };
            visitor.visit_expr(&expr);
        }
"""
replace8 = """
        for kind in exprs {
            let expr = AnalyzedExpr {
                expr: kind,
                glossa_type: crate::semantic::GlossaType::Boolean,
            };
            visit_expr(&expr, &mut usage_count);
        }
"""
code = code.replace(search8, replace8)

with open("src/tools/auditor.rs", "w") as f:
    f.write(code)
