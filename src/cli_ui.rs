use crossterm::{
    style::{Color, Print, ResetColor, SetForegroundColor},
    ExecutableCommand,
};
use std::io::{stderr, stdout, Write};

pub struct GlossaUi;

impl GlossaUi {
    pub fn new() -> Self {
        Self
    }

    pub fn success(&self, msg: &str) {
        let mut out = stderr();
        let _ = out.execute(SetForegroundColor(Color::Green));
        let _ = out.execute(Print("✓ "));
        let _ = out.execute(ResetColor);
        eprintln!("{}", msg);
    }

    #[allow(dead_code)]
    pub fn info(&self, msg: &str) {
        let mut out = stderr();
        let _ = out.execute(SetForegroundColor(Color::Blue));
        let _ = out.execute(Print("ℹ "));
        let _ = out.execute(ResetColor);
        eprintln!("{}", msg);
    }

    #[allow(dead_code)]
    pub fn error(&self, msg: &str) {
        let mut out = stderr();
        let _ = out.execute(SetForegroundColor(Color::Red));
        let _ = out.execute(Print("✗ "));
        let _ = out.execute(ResetColor);
        eprintln!("{}", msg);
    }

    pub fn prompt(&self) {
        let mut out = stdout();
        let _ = out.execute(SetForegroundColor(Color::Green));
        let _ = out.execute(Print("γλ> "));
        let _ = out.execute(ResetColor);
        let _ = out.flush();
    }

    pub fn step<F, T, E>(&self, msg: &str, f: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let mut out = stderr();

        // Print step started
        let _ = out.execute(SetForegroundColor(Color::Yellow));
        let _ = out.execute(Print("• "));
        let _ = out.execute(ResetColor);
        eprint!("{}... ", msg);
        let _ = out.flush();

        // Run the function
        let result = f();

        match &result {
            Ok(_) => {
                let _ = out.execute(SetForegroundColor(Color::Green));
                eprintln!("Done");
                let _ = out.execute(ResetColor);
            }
            Err(_) => {
                let _ = out.execute(SetForegroundColor(Color::Red));
                eprintln!("Failed");
                let _ = out.execute(ResetColor);
            }
        }

        result
    }
}
