use crossterm::{cursor, style::Stylize, terminal, ExecutableCommand};
use std::io::{self, IsTerminal, Write};
use std::time::Instant;

/// Status indicator for long-running operations
pub struct Status {
    message: String,
    start: Instant,
    is_tty: bool,
    active: bool,
}

impl Status {
    /// Create a new status indicator
    pub fn start(message: impl Into<String>) -> Self {
        let message = message.into();
        let is_tty = io::stderr().is_terminal();

        if is_tty {
            let mut stderr = io::stderr();
            // Hide cursor
            let _ = stderr.execute(cursor::Hide);
            // Print status
            eprint!("{} {}...", "⚡".yellow(), message.clone().bold());
            let _ = io::stderr().flush();
        } else {
            // For non-TTY, just print line
            eprintln!("{} {}...", "⚡".yellow(), message.clone().bold());
        }

        Self {
            message,
            start: Instant::now(),
            is_tty,
            active: true,
        }
    }

    /// Update the status message
    pub fn update(&mut self, message: impl Into<String>) {
        if !self.active {
            return;
        }

        let message = message.into();
        self.message = message.clone();

        if self.is_tty {
            let mut stderr = io::stderr();
            // Clear current line
            eprint!("\r");
            let _ = stderr.execute(terminal::Clear(terminal::ClearType::UntilNewLine));
            // Print new status
            eprint!("{} {}...", "⚡".yellow(), message.bold());
            let _ = io::stderr().flush();
        } else {
            eprintln!("{} {}...", "⚡".yellow(), message.bold());
        }
    }

    /// Mark the operation as complete success
    pub fn success(mut self) {
        if !self.active {
            return;
        }

        let duration = self.start.elapsed();
        let time_str = format!("({:.2?})", duration).dim();

        if self.is_tty {
            let mut stderr = io::stderr();
            // Clear line
            eprint!("\r");
            let _ = stderr.execute(terminal::Clear(terminal::ClearType::UntilNewLine));
            // Print success
            eprintln!("{} {} {}", "✓".green(), self.message.as_str().bold(), time_str);
            // Show cursor
            let _ = stderr.execute(cursor::Show);
        } else {
            eprintln!("{} {} {}", "✓".green(), self.message.as_str().bold(), time_str);
        }

        self.active = false;
    }

    /// Mark the operation as failed
    pub fn error(mut self, err: impl std::fmt::Display) {
        if !self.active {
            return;
        }

        if self.is_tty {
            let mut stderr = io::stderr();
            eprint!("\r");
            let _ = stderr.execute(terminal::Clear(terminal::ClearType::UntilNewLine));
            eprintln!("{} {}", "✕".red(), self.message.as_str().bold());
            // Show cursor
            let _ = stderr.execute(cursor::Show);
        } else {
            eprintln!("{} {}", "✕".red(), self.message.as_str().bold());
        }

        eprintln!("{}", err);
        self.active = false;
    }
}

impl Drop for Status {
    fn drop(&mut self) {
        if self.active && self.is_tty {
            let _ = io::stderr().execute(cursor::Show);
        }
    }
}
