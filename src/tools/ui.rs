use crossterm::{QueueableCommand, cursor, style::Stylize, terminal};
use std::io::{IsTerminal, Write};
use std::time::Instant;

/// A simple status indicator for CLI operations
pub struct Status<W: Write> {
    writer: W,
    is_tty: bool,
    start_time: Instant,
}

impl Status<std::io::Stderr> {
    /// Create a new status indicator
    pub fn new() -> Self {
        Self {
            writer: std::io::stderr(),
            is_tty: std::io::stderr().is_terminal(),
            start_time: Instant::now(),
        }
    }
}

impl<W: Write> Status<W> {
    /// Create a status indicator with a custom writer (for testing)
    pub fn new_with_writer(writer: W, is_tty: bool) -> Self {
        Self {
            writer,
            is_tty,
            start_time: Instant::now(),
        }
    }

    /// Show a starting message and reset the timer
    pub fn start(&mut self, message: &str) {
        self.start_time = Instant::now();
        if self.is_tty {
            // Clear line and print message in yellow
            let _ = self.writer.queue(cursor::MoveToColumn(0));
            let _ = self
                .writer
                .queue(terminal::Clear(terminal::ClearType::FromCursorDown));
            let _ = write!(
                self.writer,
                "{} {}",
                "⚙️".yellow(),
                message.yellow().bold()
            );
            let _ = self.writer.flush();
        } else {
            // Non-TTY: just print line
            let _ = writeln!(self.writer, "⚙️ {}", message);
        }
    }

    /// Show success message and duration
    pub fn success(&mut self, message: &str) {
        let duration = self.start_time.elapsed();
        let time_str = format!("{:.2?}", duration);

        if self.is_tty {
            let _ = self.writer.queue(cursor::MoveToColumn(0));
            let _ = self
                .writer
                .queue(terminal::Clear(terminal::ClearType::FromCursorDown));
            let _ = writeln!(
                self.writer,
                "{} {} ({})",
                "✓".green(),
                message.green().bold(),
                time_str.dim()
            );
            let _ = self.writer.flush();
        } else {
            let _ = writeln!(self.writer, "✓ {} ({})", message, time_str);
        }
    }

    /// Show failure message
    pub fn fail(&mut self, message: &str) {
        if self.is_tty {
            let _ = self.writer.queue(cursor::MoveToColumn(0));
            let _ = self
                .writer
                .queue(terminal::Clear(terminal::ClearType::FromCursorDown));
            let _ = writeln!(self.writer, "{} {}", "✕".red(), message.red().bold());
            let _ = self.writer.flush();
        } else {
            let _ = writeln!(self.writer, "✕ {}", message);
        }
    }

    // Exposed for testing to inspect buffer
    #[cfg(test)]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl Default for Status<std::io::Stderr> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_non_tty() {
        let buf = Vec::new();
        let mut status = Status::new_with_writer(buf, false);

        status.start("Starting");
        let output = String::from_utf8(status.writer.clone()).unwrap();
        assert!(output.contains("⚙️ Starting"));
        assert!(!output.contains("\x1b["));

        status.writer.clear();
        status.success("Done");
        let output = String::from_utf8(status.writer.clone()).unwrap();
        assert!(output.contains("✓ Done"));
        assert!(output.contains("("));

        status.writer.clear();
        status.fail("Failed");
        let output = String::from_utf8(status.writer.clone()).unwrap();
        assert!(output.contains("✕ Failed"));
    }

    #[test]
    fn test_status_tty() {
        let buf = Vec::new();
        let mut status = Status::new_with_writer(buf, true);

        status.start("Starting");
        let output = String::from_utf8(status.writer.clone()).unwrap();
        assert!(output.contains("Starting"));
        assert!(output.contains('\x1b'));

        status.writer.clear();
        status.success("Done");
        let output = String::from_utf8(status.writer.clone()).unwrap();
        assert!(output.contains("Done"));
        assert!(output.contains('\x1b'));

        status.writer.clear();
        status.fail("Failed");
        let output = String::from_utf8(status.writer.clone()).unwrap();
        assert!(output.contains("Failed"));
        assert!(output.contains('\x1b'));
    }
}
