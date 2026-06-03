    #[test]
    fn test_run_scholar_empty_fields_methods_functions() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("api.γλ");

        let source = "
        εἶδος Χρήστης ὁρίζειν { }.
        χαρακτήρ Εὐγενής ὁρίζειν { }.
        ";
        fs::write(&input_path, source).unwrap();

        let result = run_scholar(&input_path);
        assert!(result.is_ok());

        let output_path = input_path.with_extension("doc.md");
        assert!(output_path.exists());

        let mut f = std::fs::File::open(&output_path).unwrap();
        let mut md = String::new();
        std::io::Read::take(&mut f, 1024 * 1024 + 1)
            .read_to_string(&mut md)
            .unwrap();
        assert!(md.contains("*No fields defined.*"));
        assert!(md.contains("*No methods defined.*"));
    }

    #[test]
    fn test_run_scholar_with_functions() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("api.γλ");

        let source = "
        προσθεσις ὁρίζειν τῷ ξ ἀριθμοῦ τῷ ψ ἀριθμοῦ· δός ξ ψ ἄθροισμα.
        ";
        fs::write(&input_path, source).unwrap();

        let result = run_scholar(&input_path);
        assert!(result.is_ok());

        let output_path = input_path.with_extension("doc.md");
        assert!(output_path.exists());

        let mut f = std::fs::File::open(&output_path).unwrap();
        let mut md = String::new();
        std::io::Read::take(&mut f, 1024 * 1024 + 1)
            .read_to_string(&mut md)
            .unwrap();
        assert!(md.contains("### `προσθεσις(Ἀριθμός, Ἀριθμός) -> Ἀριθμός`"));
    }
