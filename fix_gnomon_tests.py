import os

gnomon_path = "src/tools/gnomon.rs"

with open(gnomon_path, "r") as f:
    code = f.read()

# Fix the broken tests by completely replacing them with the flattened test format
test_pattern_start = "    #[test]\n    fn test_gnomon_while_loop() {"

new_tests = """    #[test]
    fn test_gnomon_while_loop() {
        let stmt = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![],
        };
        assert_eq!(calculate_max_depth(&stmt), 1);
    }

    #[test]
    fn test_gnomon_for_loop() {
        let stmt = AnalyzedStatement::For {
            variable: SmolStr::new("x"),
            iterator: dummy_expr(),
            body: vec![],
        };
        assert_eq!(calculate_max_depth(&stmt), 1);
    }

    #[test]
    fn test_gnomon_nested_loops() {
        let inner_loop = AnalyzedStatement::For {
            variable: SmolStr::new("y"),
            iterator: dummy_expr(),
            body: vec![],
        };
        let outer_loop = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![inner_loop],
        };
        assert_eq!(calculate_max_depth(&outer_loop), 2);
    }"""

idx = code.find(test_pattern_start)
if idx != -1:
    code = code[:idx] + new_tests + "\n}\n"

with open(gnomon_path, "w") as f:
    f.write(code)

mod_path = "src/errors/mod.rs"
with open(mod_path, "r") as f:
    mod_code = f.read()

mod_code = mod_code.replace("//! This immersion helps users internalize the grammar of the language.", "//! \\> This immersion helps users internalize the grammar of the language.")

with open(mod_path, "w") as f:
    f.write(mod_code)
