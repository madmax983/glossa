import re

content = open('tests/word_order_tests.rs', 'r').read()

old_code = """    let ast = parse("ὁ ἄνθρωπος λέγει.").expect("Should parse");
    let _analyzed = analyze_program(&ast);
    // Note: analyze_program should now fail with UndefinedName because ἄνθρωπος isn't defined!
    // But it passes parsing successfully. We test that it fails in analysis.
    assert!(_analyzed.is_err());"""

new_code = """    let ast = parse("ἔστω ἄνθρωπος 1. ὁ ἄνθρωπος λέγει.").expect("Should parse");
    let _analyzed = analyze_program(&ast);
    assert!(_analyzed.is_ok());"""

if old_code in content:
    with open('tests/word_order_tests.rs', 'w') as f:
        f.write(content.replace(old_code, new_code))
    print("Replaced!")
else:
    print("Not found")
