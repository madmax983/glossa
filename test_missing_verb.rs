#[test]
fn test_missing_verb_standalone_subject() {
    let source = "α.";
    let ast = glossa::parser::parse(source).unwrap();
    let res = glossa::semantic::analyze_program(&ast);
    assert!(res.is_err());
}
#[test]
fn test_missing_verb_standalone_object() {
    let source = "τὸν ἄνθρωπον.";
    let ast = glossa::parser::parse(source).unwrap();
    let res = glossa::semantic::analyze_program(&ast);
    assert!(res.is_err());
}
#[test]
fn test_missing_verb_pattern_arm() {
    let source = "κατὰ ξ· μηδὲν ᾖ, «μηδέν» λέγε· ἄλλο ᾖ, «ἄλλο» λέγε.";
    let ast = glossa::parser::parse(source).unwrap();
    let res = glossa::semantic::analyze_program(&ast);
    assert!(res.is_err());
}
