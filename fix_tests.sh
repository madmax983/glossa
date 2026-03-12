sed -i 's/use crate::semantic::analyzer::SemanticAnalyzer;//g' src/semantic/control_flow.rs
sed -i 's/let mut analyzer = SemanticAnalyzer::new();//g' src/semantic/control_flow.rs
sed -i 's/, \&mut analyzer//g' src/semantic/control_flow.rs
