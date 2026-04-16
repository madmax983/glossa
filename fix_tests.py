import sys

def main():
    with open('src/tools/simulator.rs', 'r') as f:
        content = f.read()

    tests_str = """#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_simulator_success() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("simulator_test.γλ");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("«χαῖρε κόσμε» λέγε.\\n".as_bytes()).unwrap();
        }

        let result = run_simulator(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_simulator_file_not_found() {
        let path = Path::new("non_existent_file.γλ");
        let result = run_simulator(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("οὐχ εὑρέθη"));
    }

    #[test]
    fn test_run_simulator_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("parse_error.γλ");
        std::fs::write(&input_path, b"invalid syntax").unwrap();

        let result = run_simulator(&input_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_simulator_semantic_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("semantic_error.γλ");
        // Valid syntax, but 'ψ' is not defined, causing a semantic analysis error
        std::fs::write(&input_path, "ψ 10 γίγνεται.").unwrap();

        let result = run_simulator(&input_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("οὐκ οἶδα τὸ ὄνομα"));
    }

    #[test]
    fn test_run_simulator_not_implemented() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("simulator_not_impl.γλ");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&input_path).unwrap();
            // Struct definitions are not yet supported by the Interpreter
            f.write_all("εἶδος Χ ὁρίζειν { χ ἀριθμοῦ. }.\\n".as_bytes()).unwrap();
        }

        // It should succeed but report Partial Execution
        let result = run_simulator(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_simulator_runtime_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("simulator_runtime_error.γλ");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&input_path).unwrap();
            // Valid parse, valid semantics, but runtime error (divide by zero)
            f.write_all("1 0 μέρος λέγε.\\n".as_bytes()).unwrap();
        }

        let result = run_simulator(&input_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Runtime error"));
    }

    #[test]
    fn test_run_simulator_empty_output() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("simulator_empty_output.γλ");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&input_path).unwrap();
            // Valid parse, valid semantics, no output
            f.write_all("ξ 10 ἔστω.\\n".as_bytes()).unwrap();
        }

        let result = run_simulator(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_simulator_file_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("file_error.γλ");
        std::fs::write(&input_path, "ξ 10 ἔστω.").unwrap();

        // Simulate a file error (e.g., trying to run a directory as a file)
        let dir_path = dir.path().join("a_directory.γλ");
        std::fs::create_dir(&dir_path).unwrap();

        let result = run_simulator(&dir_path);
        assert!(result.is_err());
    }
}
"""

    old_tests_str = """#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_simulator_success() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("simulator_test.γλ");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("«χαῖρε κόσμε» λέγε.\\n".as_bytes()).unwrap();
        }

        let result = run_simulator(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_simulator_file_not_found() {
        let path = Path::new("non_existent_file.γλ");
        let result = run_simulator(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("οὐχ εὑρέθη"));
    }

    #[test]
    fn test_run_simulator_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("parse_error.γλ");
        std::fs::write(&input_path, b"invalid syntax").unwrap();

        let result = run_simulator(&input_path);
        assert!(result.is_err());
    }
}"""

    content = content.replace(old_tests_str, tests_str)

    with open('src/tools/simulator.rs', 'w') as f:
        f.write(content)

if __name__ == "__main__":
    main()
