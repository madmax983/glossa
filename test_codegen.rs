#[derive(Debug, Clone)]
enum AnalyzedExpr {
    BinOp(Box<AnalyzedExpr>, Box<AnalyzedExpr>),
    Number(i32),
}
