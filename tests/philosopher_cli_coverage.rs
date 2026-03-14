#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;
    use tempfile::tempdir;

    #[test]
    fn test_philosopher_cli_integration() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test.gl");
        let source_code = "
            πολλαπαραμ ὁρίζειν τῷ a τῷ b τῷ c τῷ d·
                a λέγε.
        ";
        fs::write(&source_path, source_code).unwrap();

        let output = Command::new(env!("CARGO"))
            .arg("run")
            .arg("--features")
            .arg("nova")
            .arg("--")
            .arg("philosopher")
            .arg(&source_path)
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Μηδὲν ἄγαν (Nothing in excess)"));
    }

    #[test]
    fn test_philosopher_cli_labyrinth_and_moderation() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test2.gl");
        let source_code = "
            τολονγ ὁρίζειν τῷ α·
                α λέγε· α λέγε· α λέγε· α λέγε· α λέγε· α λέγε·
                α λέγε· α λέγε· α λέγε· α λέγε· α λέγε· α λέγε.
        ";
        fs::write(&source_path, source_code).unwrap();

        let output = Command::new(env!("CARGO"))
            .arg("run")
            .arg("--features")
            .arg("nova")
            .arg("--")
            .arg("philosopher")
            .arg(&source_path)
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Μέτρον ἄριστον (Moderation is best)"));
    }

    #[test]
    fn test_philosopher_cli_perfect() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test3.gl");
        let source_code = "
            α 5 ἔστω.
        ";
        fs::write(&source_path, source_code).unwrap();

        let output = Command::new(env!("CARGO"))
            .arg("run")
            .arg("--features")
            .arg("nova")
            .arg("--")
            .arg("philosopher")
            .arg(&source_path)
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Aristotle would be proud"));
    }
}
