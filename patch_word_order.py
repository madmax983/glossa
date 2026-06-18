import re

with open('tests/word_order_tests.rs', 'r') as f:
    content = f.read()

# Ah! `ὁ ἄνθρωπος λέγει.` evaluates to `UndefinedName { name: "ανθρωπος" }`
# Because `ἄνθρωπος` is evaluated as an `Undefined` variable by `try_print_default`!
# Why did this pass before? Because `try_print_default` was silently ignoring it!
# If `ἄνθρωπος` is not defined, we SHOULD throw `UndefinedName`!
# In `test_article_disambiguation_context`, it is just testing that the parsing/analysis doesn't panic.
# But now it throws a semantic error, which is correct because `ἄνθρωπος` IS undefined!
# Let's add `ἄνθρωπος 1 ἔστω.` before it.
replacement = """fn test_article_disambiguation_context() {
    // ὁ ἄνθρωπος should be recognized as masculine nominative singular
    // τὸν λόγον should be recognized as masculine accusative singular
    // These don't produce Rust code yet, but they should parse without error
    let ast = parse("ἄνθρωπος 1 ἔστω. ὁ ἄνθρωπος λέγει.").expect("Should parse");
    let _analyzed = analyze_program(&ast).expect("Should analyze");
}"""

content = re.sub(
    r'fn test_article_disambiguation_context\(\) \{\n.*?let _analyzed = analyze_program\(&ast\)\.expect\("Should analyze"\);\n\}',
    replacement,
    content,
    flags=re.DOTALL
)

with open('tests/word_order_tests.rs', 'w') as f:
    f.write(content)
