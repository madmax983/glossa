//! Trait definitions for semantic analysis

use crate::ast::Statement;
use crate::errors::GlossaError;
use crate::semantic::{AnalyzedStatement, Scope};

/// Abstract trait for analyzing statements.
/// This breaks the circular dependency between the orchestrator (`mod.rs`/`analyzer.rs`)
/// and the submodules (`control_flow.rs`, `declarations.rs`).
pub trait StatementAnalyzer {
    /// Semantically analyze a single statement and update the scope environment.
    fn analyze_statement(
        &mut self,
        stmt: &Statement,
        scope: &mut Scope,
    ) -> Result<Vec<AnalyzedStatement>, GlossaError>;
}
