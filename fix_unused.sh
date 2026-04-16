#!/bin/bash
sed -i 's/use glossa::semantic::{/use glossa::semantic::{/g' tests/semantic_model_coverage_tests.rs
sed -i 's/    AnalyzedExpr, AnalyzedExprKind, AnalyzedMethod, AnalyzedStatement, CaptureMode, GlossaType,/    AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, CaptureMode, GlossaType,/g' tests/semantic_model_coverage_tests.rs
sed -i '/use std::collections::HashMap;/d' tests/semantic_model_coverage_tests.rs
