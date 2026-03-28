import sys

def main():
    content = open("src/experimental/simulator.rs").read()
    if "let _ = interp.run(&current_prog);" not in content:
        content = content.replace("let result = interp.run(&current_prog);\n        \n        let state_desc = match result {\n            Ok(_) => {\n                match stmt {", "let _ = interp.run(&current_prog);\n        \n        let state_desc = match stmt {")
        content = content.replace("                    },\n                    _ => \"Ok\".to_string()\n                }\n            },\n            Err(e) => format!(\"Error: {:?}\", e),\n        };", "                    },\n                    _ => \"Ok\".to_string()\n                };")
        open("src/experimental/simulator.rs", "w").write(content)
        print("Patched simulator.rs")

if __name__ == "__main__":
    main()
