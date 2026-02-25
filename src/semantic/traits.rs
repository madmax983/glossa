use crate::ast::Statement;
use crate::errors::GlossaError;
use crate::semantic::model::AnalyzedStatement;
use crate::semantic::resolver::Scope;

/// Trait for analyzing statements recursively
///
/// This trait decouples `analyze_control_flow` and `analyze_statement`, breaking
/// the circular dependency between `semantic/mod.rs` (logic) and `semantic/control_flow.rs`.
pub trait StatementAnalyzer {
    /// Analyze a single statement using the analyzer's logic
    fn analyze(
        &mut self,
        stmt: &Statement,
        scope: &mut Scope,
    ) -> Result<Vec<AnalyzedStatement>, GlossaError>;
}
