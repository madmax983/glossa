with open("src/tools/scholar.rs", "r") as f:
    content = f.read()

search = """    #[test]
    fn test_scholar() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_scholar.gl");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            f.write_all(b"\\xce\\xb5\\xcf\\x83\\xcf\\x84\\xcf\\x89 \\xce\\xbe 10.").unwrap();
        }

        let result = run_scholar(&file_path);
        assert!(result.is_ok());
    }"""

replace = """    #[test]
    fn test_scholar() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_scholar.gl");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            f.write_all("ἔστω ξ 10.".as_bytes()).unwrap();
        }

        let result = run_scholar(&file_path);
        assert!(result.is_ok());
    }"""

content = content.replace(search, replace)

with open("src/tools/scholar.rs", "w") as f:
    f.write(content)
