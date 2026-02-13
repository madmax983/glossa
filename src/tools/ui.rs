use crossterm::{QueueableCommand, cursor, style::Stylize, terminal};
use std::io::{IsTerminal, Write};
use std::time::Instant;

/// A simple status indicator for CLI operations
pub struct Status {
    is_tty: bool,
    start_time: Instant,
}

impl Status {
    /// Create a new status indicator
    pub fn new() -> Self {
        Self {
            is_tty: std::io::stderr().is_terminal(),
            start_time: Instant::now(),
        }
    }

    /// Show a starting message and reset the timer
    pub fn start(&mut self, message: &str) {
        self.start_time = Instant::now();
        if self.is_tty {
            let mut stderr = std::io::stderr();
            // Clear line and print message in yellow
            let _ = stderr.queue(cursor::MoveToColumn(0));
            let _ = stderr.queue(terminal::Clear(terminal::ClearType::FromCursorDown));
            let _ = write!(stderr, "{} {}", "⚙️".yellow(), message.yellow().bold());
            let _ = stderr.flush();
        } else {
            // Non-TTY: just print line
            eprintln!("⚙️ {}", message);
        }
    }

    /// Show success message and duration
    pub fn success(&self, message: &str) {
        let duration = self.start_time.elapsed();
        let time_str = format!("{:.2?}", duration);

        if self.is_tty {
            let mut stderr = std::io::stderr();
            let _ = stderr.queue(cursor::MoveToColumn(0));
            let _ = stderr.queue(terminal::Clear(terminal::ClearType::FromCursorDown));
            let _ = writeln!(
                stderr,
                "{} {} ({})",
                "✓".green(),
                message.green().bold(),
                time_str.dim()
            );
            let _ = stderr.flush();
        } else {
            eprintln!("✓ {} ({})", message, time_str);
        }
    }

    /// Show failure message
    pub fn fail(&self, message: &str) {
        if self.is_tty {
            let mut stderr = std::io::stderr();
            let _ = stderr.queue(cursor::MoveToColumn(0));
            let _ = stderr.queue(terminal::Clear(terminal::ClearType::FromCursorDown));
            let _ = writeln!(stderr, "{} {}", "✕".red(), message.red().bold());
            let _ = stderr.flush();
        } else {
            eprintln!("✕ {}", message);
        }
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::new()
    }
}
