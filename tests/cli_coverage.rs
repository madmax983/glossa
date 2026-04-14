#[cfg(test)]
mod tests {
    use glossa::tools::{Cli, Commands};

    #[test]
    fn coverage() {
        let _ = Cli { file: None, command: None };
        let _ = Commands::Repl;
    }
}
