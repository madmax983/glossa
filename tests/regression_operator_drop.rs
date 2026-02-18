// #[cfg(test)]
// mod tests {
//     use glossa::morphology::lexicon::BinaryOp;
//     use glossa::semantic::Literal;
//     use glossa::semantic::expressions::build_expressions_from_literals_and_ops;
//
//     #[test]
//     fn test_dropped_operator() {
//         // Case: 1 + 2 +
//         // Literals: [1, 2]
//         // Operators: [Add, Add]
//         // Expected: Should return Error due to insufficient literals
//
//         let literals = vec![Literal::Number(1), Literal::Number(2)];
//         let operators = vec![BinaryOp::Add, BinaryOp::Add];
//
//         let result = build_expressions_from_literals_and_ops(&literals, &operators);
//
//         assert!(
//             result.is_err(),
//             "Expected error for dangling operator, got {:?}",
//             result
//         );
//
//         let err = result.unwrap_err();
//         assert!(
//             err.to_string().contains("Insufficient literals"),
//             "Unexpected error message: {}",
//             err
//         );
//     }
// }
