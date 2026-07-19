#[cfg(feature = "nova")]
#[cfg(test)]
mod tests {
    use glossa::tools::emissary::run_emissary;
    use std::path::Path;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_emissary_valid_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "εἶδος Χρήστης ὁρίζειν {{\n    ὄνομα ὀνόματος.\n}}.").unwrap();

        let result = run_emissary(file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_emissary_missing_file() {
        let path = Path::new("non_existent_file.γλ");
        let result = run_emissary(path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Ἀρχεῖον οὐχ εὑρέθη: non_existent_file.γλ");
    }
}
