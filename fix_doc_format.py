with open("src/tools/scholar.rs", "r") as f:
    content = f.read()

content = content.replace("}\n}\n\n    #[test]", "    #[test]")
content = content.replace("    #[test]\n    fn test_run_scholar_with_functions() {", "    #[test]\n    fn test_run_scholar_with_functions() {")

with open("src/tools/scholar.rs", "w") as f:
    f.write(content)
