import re

with open('src/semantic/model.rs', 'r') as f:
    content = f.read()

content = content.replace("#[derive(Clone)]\npub enum AnalyzedStatement", "pub enum AnalyzedStatement")

clone_drop_impl = """
impl Clone for AnalyzedStatement {
    fn clone(&self) -> Self {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            AnalyzedStatement::Binding { name, value, mutable } => AnalyzedStatement::Binding { name: name.clone(), value: value.clone(), mutable: *mutable },
            AnalyzedStatement::Assignment { name, value } => AnalyzedStatement::Assignment { name: name.clone(), value: value.clone() },
            AnalyzedStatement::Print(exprs) => AnalyzedStatement::Print(exprs.clone()),
            AnalyzedStatement::Expression(exprs) => AnalyzedStatement::Expression(exprs.clone()),
            AnalyzedStatement::Query(exprs) => AnalyzedStatement::Query(exprs.clone()),
            AnalyzedStatement::If { condition, then_body, else_body } => AnalyzedStatement::If { condition: condition.clone(), then_body: then_body.clone(), else_body: else_body.clone() },
            AnalyzedStatement::While { condition, body } => AnalyzedStatement::While { condition: condition.clone(), body: body.clone() },
            AnalyzedStatement::For { item_name, iterable, body } => AnalyzedStatement::For { item_name: item_name.clone(), iterable: iterable.clone(), body: body.clone() },
            AnalyzedStatement::Match { scrutinee, arms } => AnalyzedStatement::Match { scrutinee: scrutinee.clone(), arms: arms.clone() },
            AnalyzedStatement::Break => AnalyzedStatement::Break,
            AnalyzedStatement::Continue => AnalyzedStatement::Continue,
            AnalyzedStatement::Return { value } => AnalyzedStatement::Return { value: value.clone() },
            AnalyzedStatement::FunctionDef { name, params, body, return_type } => AnalyzedStatement::FunctionDef { name: name.clone(), params: params.clone(), body: body.clone(), return_type: return_type.clone() },
            AnalyzedStatement::TypeDefinition { name, fields } => AnalyzedStatement::TypeDefinition { name: name.clone(), fields: fields.clone() },
            AnalyzedStatement::TraitDefinition { name, methods } => AnalyzedStatement::TraitDefinition { name: name.clone(), methods: methods.clone() },
            AnalyzedStatement::TraitImplementation { trait_name, type_name, methods } => AnalyzedStatement::TraitImplementation { trait_name: trait_name.clone(), type_name: type_name.clone(), methods: methods.clone() },
            AnalyzedStatement::TestDeclaration { name, body } => AnalyzedStatement::TestDeclaration { name: name.clone(), body: body.clone() },
        })
    }
}

impl Drop for AnalyzedStatement {
    fn drop(&mut self) {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            match self {
                AnalyzedStatement::If { condition, then_body, else_body } => {
                    let _ = std::mem::replace(condition, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean });
                    let _ = std::mem::take(then_body);
                    let _ = std::mem::take(else_body);
                }
                AnalyzedStatement::While { condition, body } => {
                    let _ = std::mem::replace(condition, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean });
                    let _ = std::mem::take(body);
                }
                AnalyzedStatement::For { iterable, body, .. } => {
                    let _ = std::mem::replace(iterable, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean });
                    let _ = std::mem::take(body);
                }
                AnalyzedStatement::Match { scrutinee, arms } => {
                    let _ = std::mem::replace(scrutinee, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean });
                    let _ = std::mem::take(arms);
                }
                AnalyzedStatement::FunctionDef { body, .. } => {
                    let _ = std::mem::take(body);
                }
                AnalyzedStatement::TestDeclaration { body, .. } => {
                    let _ = std::mem::take(body);
                }
                _ => {}
            }
        })
    }
}
"""

content = content.replace("    /// pub body: Option<Vec<AnalyzedStatement>>,\n    /// }\n", "    /// pub body: Option<Vec<AnalyzedStatement>>,\n    /// }\n" + clone_drop_impl)


with open('src/semantic/model.rs', 'w') as f:
    f.write(content)
