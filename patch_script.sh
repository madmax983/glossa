sed -i 's/use super::{AnalyzedStatement, GlossaType, Scope, StatementAnalyzer, assemble_statement};/use super::{AnalyzedStatement, GlossaType, Scope, assemble_statement};/g' src/semantic/analyzer.rs
sed -i 's/pub struct SemanticAnalyzer;//g' src/semantic/analyzer.rs
sed -i 's/impl SemanticAnalyzer {//g' src/semantic/analyzer.rs
sed -i 's/    pub fn new() -> Self {//g' src/semantic/analyzer.rs
sed -i 's/        Self//g' src/semantic/analyzer.rs
sed -i 's/    }//g' src/semantic/analyzer.rs
sed -i 's/impl Default for SemanticAnalyzer {//g' src/semantic/analyzer.rs
sed -i 's/    fn default() -> Self {//g' src/semantic/analyzer.rs
sed -i 's/        Self::new()//g' src/semantic/analyzer.rs
sed -i 's/impl StatementAnalyzer for SemanticAnalyzer {//g' src/semantic/analyzer.rs
sed -i 's/    fn analyze_statement(/pub fn analyze_statement(/g' src/semantic/analyzer.rs
sed -i '/\&mut self,/d' src/semantic/analyzer.rs
sed -i 's/parse_function_definition(stmt, scope, self)?/parse_function_definition(stmt, scope)?/g' src/semantic/analyzer.rs
sed -i 's/analyze_control_flow(stmt, scope, self)?/analyze_control_flow(stmt, scope)?/g' src/semantic/analyzer.rs
sed -i 's/self.analyze_statement/analyze_statement/g' src/semantic/analyzer.rs
sed -i 's/let mut analyzer = SemanticAnalyzer::new();//g' src/semantic/analyzer.rs
sed -i 's/analyzer.analyze_statement/analyze_statement/g' src/semantic/analyzer.rs
sed -i 's/\&mut analyzer,/\&mut analyzer,/g' src/semantic/analyzer.rs
sed -i 's/analyze_type_definition(\n                type_def,\n                &mut scope,\n                &mut analyzer,\n            )?/analyze_type_definition(type_def, &mut scope)?/g' src/semantic/analyzer.rs
sed -i 's/analyze_trait_definition(\n                trait_def,\n                &mut scope,\n                &mut analyzer,\n            )?/analyze_trait_definition(trait_def, &mut scope)?/g' src/semantic/analyzer.rs
sed -i 's/analyze_trait_impl(trait_impl, &mut scope, &mut analyzer)?/analyze_trait_impl(trait_impl, &mut scope)?/g' src/semantic/analyzer.rs
sed -i 's/analyze_test_declaration(\n                test_decl,\n                &mut scope,\n                &mut analyzer,\n            )?/analyze_test_declaration(test_decl, &mut scope)?/g' src/semantic/analyzer.rs

sed -i 's/use super::{/use super::{analyzer::analyze_statement, /g' src/semantic/control_flow.rs
sed -i 's/    AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType, Scope, StatementAnalyzer,/    AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType, Scope,/g' src/semantic/control_flow.rs
sed -i '/analyzer: \&mut impl StatementAnalyzer,/d' src/semantic/control_flow.rs
sed -i 's/analyzer\.analyze_statement(/analyze_statement(/g' src/semantic/control_flow.rs
sed -i 's/parse_conditional(stmt, scope, analyzer, 0)/parse_conditional(stmt, scope, 0)/g' src/semantic/control_flow.rs
sed -i 's/parse_while_loop(stmt, scope, analyzer)/parse_while_loop(stmt, scope)/g' src/semantic/control_flow.rs
sed -i 's/parse_for_range_loop(stmt, scope, analyzer)/parse_for_range_loop(stmt, scope)/g' src/semantic/control_flow.rs
sed -i 's/parse_for_iteration_loop(stmt, scope, analyzer)/parse_for_iteration_loop(stmt, scope)/g' src/semantic/control_flow.rs
sed -i 's/parse_match_expression(stmt, scope, analyzer)/parse_match_expression(stmt, scope)/g' src/semantic/control_flow.rs
sed -i 's/parse_conditional(\&elif_stmt, scope, analyzer, depth + 1)?/parse_conditional(\&elif_stmt, scope, depth + 1)?/g' src/semantic/control_flow.rs

sed -i 's/use super::{AnalyzedMethod, AnalyzedStatement, GlossaType, Scope, StatementAnalyzer};/use super::{analyzer::analyze_statement, AnalyzedMethod, AnalyzedStatement, GlossaType, Scope};/g' src/semantic/declarations.rs
sed -i '/_analyzer: \&mut impl StatementAnalyzer,/d' src/semantic/declarations.rs
sed -i '/analyzer: \&mut impl StatementAnalyzer,/d' src/semantic/declarations.rs
sed -i 's/analyzer\.analyze_statement(/analyze_statement(/g' src/semantic/declarations.rs
