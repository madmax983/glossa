import sys

def main():
    content = open("tests/simulator_coverage.rs").read()
    if "let _program = analyze_program(&ast).unwrap();" in content:
        content = content.replace("let _program = analyze_program(&ast).unwrap();", "let program = analyze_program(&ast).unwrap();\n    let _ = glossa::experimental::simulator::run_simulation(&program);")
        open("tests/simulator_coverage.rs", "w").write(content)
        print("Patched simulator_coverage.rs")

if __name__ == "__main__":
    main()
