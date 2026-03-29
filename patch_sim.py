import sys

def main():
    content = open("src/experimental/simulator.rs").read()
    if "_ => \"Execute Statement\".to_string()," in content:
        content = content.replace("            AnalyzedStatement::Expression(_) => \"Evaluate Expression\".to_string(),\n            AnalyzedStatement::Print(_) => \"Print\".to_string(),\n            _ => \"Execute Statement\".to_string(),\n        };", "            AnalyzedStatement::Expression(_) => \"Evaluate Expression\".to_string(),\n            AnalyzedStatement::Print(_) => \"Print\".to_string(),\n            AnalyzedStatement::If { .. } => \"If Condition\".to_string(),\n            AnalyzedStatement::While { .. } => \"While Loop\".to_string(),\n            AnalyzedStatement::For { .. } => \"For Loop\".to_string(),\n            AnalyzedStatement::FunctionDef { .. } => \"Define Function\".to_string(),\n            AnalyzedStatement::Return { .. } => \"Return\".to_string(),\n            AnalyzedStatement::TypeDefinition { .. } => \"Define Type\".to_string(),\n            AnalyzedStatement::TraitDefinition { .. } => \"Define Trait\".to_string(),\n            AnalyzedStatement::TraitImplementation { .. } => \"Implement Trait\".to_string(),\n            AnalyzedStatement::TestDeclaration { .. } => \"Declare Test\".to_string(),\n            AnalyzedStatement::Break => \"Break\".to_string(),\n            AnalyzedStatement::Continue => \"Continue\".to_string(),\n            _ => \"Execute Statement\".to_string(),\n        };")

        open("src/experimental/simulator.rs", "w").write(content)
        print("Patched simulator.rs")

if __name__ == "__main__":
    main()
