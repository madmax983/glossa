use crossterm::{ExecutableCommand, cursor, style::Stylize, terminal};
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
        let is_tty = io::stderr().is_terminal();
        Self::new(message, is_tty)
    }

    /// Internal constructor for testing
    fn new(message: impl Into<String>, is_tty: bool) -> Self {
        let status = Self {
            message: message.into(),
            start: Instant::now(),
            is_tty,
            active: true,
        };
        status.print_running(false);
        status
    }

    /// Update the status message
    pub fn update(&mut self, message: impl Into<String>) {
        if !self.active {
            return;
        }
        self.message = message.into();
        self.print_running(true);
    }

    /// Mark the operation as complete success
    pub fn success(mut self) {
        if !self.active {
            return;
        }

        let duration = self.start.elapsed();
        let time_str = format!("({:.2?})", duration).dim();
        let msg = format!("{} {}", self.message.as_str().bold(), time_str);

        self.print_done("✓".green(), &msg);
        self.active = false;
    }

    /// Mark the operation as failed
    pub fn error(mut self, err: impl std::fmt::Display) {
        if !self.active {
            return;
        }

        let msg = self.message.as_str().bold().to_string();
        self.print_done("✕".red(), &msg);
        eprintln!("{}", err);
        self.active = false;
    }

    fn print_running(&self, clear: bool) {
        let symbol = "⚡".yellow();
        let msg = format!("{}...", self.message.as_str().bold());

        if self.is_tty {
            let mut stderr = io::stderr();
            if clear {
                eprint!("\r");
                let _ = stderr.execute(terminal::Clear(terminal::ClearType::UntilNewLine));
            } else {
                let _ = stderr.execute(cursor::Hide);
            }
            eprint!("{} {}", symbol, msg);
            let _ = io::stderr().flush();
        } else {
            eprintln!("{} {}", symbol, msg);
        }
    }

    fn print_done(&self, symbol: impl std::fmt::Display, message: &str) {
        if self.is_tty {
            let mut stderr = io::stderr();
            eprint!("\r");
            let _ = stderr.execute(terminal::Clear(terminal::ClearType::UntilNewLine));
            eprintln!("{} {}", symbol, message);
            let _ = stderr.execute(cursor::Show);
        } else {
            eprintln!("{} {}", symbol, message);
        }
    }
}

impl Drop for Status {
    fn drop(&mut self) {
        if self.active && self.is_tty {
            let _ = io::stderr().execute(cursor::Show);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_tty_success() {
        let status = Status::new("Testing TTY", true);
        // Should execute TTY branches
        status.success();
    }

    #[test]
    fn test_status_tty_update() {
        let mut status = Status::new("Testing TTY Update", true);
        status.update("Updated");
        status.success();
    }

    #[test]
    fn test_status_tty_error() {
        let status = Status::new("Testing TTY Error", true);
        status.error("Something went wrong");
    }

    #[test]
    fn test_status_no_tty_success() {
        let status = Status::new("Testing No-TTY", false);
        // Should execute non-TTY branches
        status.success();
    }

    #[test]
    fn test_status_no_tty_update() {
        let mut status = Status::new("Testing No-TTY Update", false);
        status.update("Updated");
        status.success();
    }

    #[test]
    fn test_status_no_tty_error() {
        let status = Status::new("Testing No-TTY Error", false);
        status.error("Something went wrong");
    }

    #[test]
    fn test_status_drop() {
        {
            let _status = Status::new("Testing Drop", true);
            // Should execute Drop (show cursor)
        }
    }
}
