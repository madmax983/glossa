//! The Stage (UI) Tool
//!
//! This module provides terminal user interface components for the CLI.
//! It handles status indicators, spinners, and stylized output.
//!
//! # The Philosophy: "The Show Must Go On"
//!
//! A compiler should not be a black box that hangs silently. It should provide
//! feedback on what it's doing.
//!
//! # Key Components
//!
//! *   **[`Status`]**: A context manager for long-running operations. It displays a
//!     spinner or status message and handles cleanup (success/failure) automatically.
//!
//! # CI Friendliness
//!
//! The module detects if it is running in a TTY (interactive terminal).
//! *   **TTY**: Uses animated spinners and rewriting lines (`\r`).
//! *   **No-TTY (CI)**: Uses simple log lines to avoid cluttering build logs with escape codes.

use crossterm::{ExecutableCommand, cursor, style::Stylize, terminal};
use std::io::{self, IsTerminal, Write};
use std::time::Instant;

/// Status indicator for long-running operations
pub struct Status {
    message: String,
    symbol: String,
    start: Instant,
    is_tty: bool,
    active: bool,
}

impl Status {
    /// Create a new status indicator with default symbol
    #[allow(dead_code)]
    pub fn start(message: impl Into<String>) -> Self {
        Self::start_with_symbol(message, "⚡")
    }

    /// Create a new status indicator with a custom symbol
    pub fn start_with_symbol(message: impl Into<String>, symbol: impl Into<String>) -> Self {
        let is_tty = io::stderr().is_terminal();
        Self::new(message, symbol, is_tty)
    }

    /// Internal constructor for testing
    fn new(message: impl Into<String>, symbol: impl Into<String>, is_tty: bool) -> Self {
        let status = Self {
            message: message.into(),
            symbol: symbol.into(),
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
        // Apply yellow to symbol for consistency, assuming standard terminal behavior
        let symbol = self.symbol.as_str().yellow();
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
        let status = Status::new("Testing TTY", "⚡", true);
        // Should execute TTY branches
        status.success();
    }

    #[test]
    fn test_status_tty_update() {
        let mut status = Status::new("Testing TTY Update", "⚡", true);
        status.update("Updated");
        status.success();
    }

    #[test]
    fn test_status_tty_error() {
        let status = Status::new("Testing TTY Error", "⚡", true);
        status.error("Something went wrong");
    }

    #[test]
    fn test_status_no_tty_success() {
        let status = Status::new("Testing No-TTY", "⚡", false);
        // Should execute non-TTY branches
        status.success();
    }

    #[test]
    fn test_status_no_tty_update() {
        let mut status = Status::new("Testing No-TTY Update", "⚡", false);
        status.update("Updated");
        status.success();
    }

    #[test]
    fn test_status_no_tty_error() {
        let status = Status::new("Testing No-TTY Error", "⚡", false);
        status.error("Something went wrong");
    }

    #[test]
    fn test_status_custom_symbol() {
        let status = Status::new("Custom Symbol", "🚀", false);
        status.success();
    }

    #[test]
    fn test_status_drop() {
        {
            let _status = Status::new("Testing Drop", "⚡", true);
            // Should execute Drop (show cursor)
        }
    }
}
