    #[test]
    fn test_commands_input_path_all() {
        let p = PathBuf::from("test.γλ");

        let commands_with_input = vec![
            Commands::Run { input: p.clone() },
            Commands::Report { input: p.clone() },
            Commands::Labyrinth { input: p.clone() },
            Commands::Build {
                input: p.clone(),
                output: None,
            },
            Commands::Check { input: p.clone() },
            Commands::Highlight { input: p.clone() },
            Commands::Bard { input: p.clone() },
            Commands::Test { input: p.clone() },
            Commands::Mosaic { input: p.clone() },
            Commands::Map { input: p.clone() },
            Commands::Weave { input: p.clone() },
            Commands::Alchemist { input: p.clone() },
            Commands::Papyrus { input: p.clone() },
            Commands::Audit { input: p.clone() },
            Commands::Scholar { input: p.clone() },
        ];

        for cmd in commands_with_input {
            assert_eq!(cmd.input_path(), Some(&p));
        }

        let commands_without_input = vec![
            Commands::Mentor,
            Commands::Repl,
            Commands::Lookup {
                word: "test".to_string(),
            },
            Commands::Catalog,
        ];

        for cmd in commands_without_input {
            assert_eq!(cmd.input_path(), None);
        }
    }
