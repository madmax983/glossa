use crossterm::{QueueableCommand, cursor, style::Stylize, terminal};
use std::io::{IsTerminal, Write};
use std::time::Instant;

/// A simple status indicator for CLI operations
pub struct Status {
    writer: Box<dyn Write + Send>,
    is_tty: bool,
    start_time: Instant,
}

impl Status {
    /// Create a new status indicator writing to stderr
    pub fn new() -> Self {
        Self {
            writer: Box::new(std::io::stderr()),
            is_tty: std::io::stderr().is_terminal(),
            start_time: Instant::now(),
        }
    }

    /// Create a status indicator with a custom writer (for testing)
    pub fn new_with_writer(writer: Box<dyn Write + Send>, is_tty: bool) -> Self {
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
}

impl Default for Status {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // A thread-safe buffer for testing
    #[derive(Clone)]
    struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

    impl SharedBuffer {
        fn new() -> Self {
            Self(Arc::new(Mutex::new(Vec::new())))
        }

        fn content(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }

        fn clear(&self) {
            self.0.lock().unwrap().clear();
        }
    }

    impl Write for SharedBuffer {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_status_non_tty() {
        let buf = SharedBuffer::new();
        let mut status = Status::new_with_writer(Box::new(buf.clone()), false);

        status.start("Starting");
        let output = buf.content();
        assert!(output.contains("⚙️ Starting"));
        assert!(!output.contains("\x1b["));

        buf.clear();
        status.success("Done");
        let output = buf.content();
        assert!(output.contains("✓ Done"));
        assert!(output.contains("("));

        buf.clear();
        status.fail("Failed");
        let output = buf.content();
        assert!(output.contains("✕ Failed"));
    }

    #[test]
    fn test_status_tty() {
        let buf = SharedBuffer::new();
        let mut status = Status::new_with_writer(Box::new(buf.clone()), true);

        status.start("Starting");
        let output = buf.content();
        assert!(output.contains("Starting"));
        assert!(output.contains('\x1b'));

        buf.clear();
        status.success("Done");
        let output = buf.content();
        assert!(output.contains("Done"));
        assert!(output.contains('\x1b'));

        buf.clear();
        status.fail("Failed");
        let output = buf.content();
        assert!(output.contains("Failed"));
        assert!(output.contains('\x1b'));
    }
}
