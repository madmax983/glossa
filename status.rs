impl Status {
    /// Create a new status indicator with default symbol (⚡)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::tools::ui::Status;
    /// let status = Status::start("Compiling...");
    /// status.success();
    /// ```
    pub fn start(message: impl Into<String>) -> Self {
        Self::start_with_symbol(message, "⚡")
    }

    /// Create a new status indicator with a custom symbol
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::tools::ui::Status;
    /// let status = Status::start_with_symbol("Testing", "🧪");
    /// status.success();
    /// ```
    pub fn start_with_symbol(message: impl Into<String>, symbol: impl Into<String>) -> Self {
        let is_tty = io::stderr().is_terminal();
        Self::new(message, symbol, is_tty)
    }

    /// Internal constructor for testing
    fn new(message: impl Into<String>, symbol: impl Into<String>, is_tty: bool) -> Self {
        let msg = message.into();
        let sym = symbol.into();
        let (tx, thread) = if is_tty {
            let (tx, rx) = mpsc::channel::<(bool, Option<String>)>();
            let thread_msg = msg.clone();
            let thread_sym = sym.clone();
            let handle = thread::spawn(move || {
                let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                let mut current_msg = thread_msg;
                let mut i = 0;
                loop {
                    // Drain all messages to prevent backlog
                    let mut should_stop = false;
                    while let Ok((stop, new_msg)) = rx.try_recv() {
                        if stop {
                            should_stop = true;
                            break;
                        }
                        if let Some(m) = new_msg {
                            current_msg = m;
                        }
                    }

                    if should_stop {
                        break;
                    }

                    let frame = frames[i % frames.len()];
                    let symbol_colored = thread_sym.as_str().yellow();
                    let msg_bold = format!("{}...", current_msg.as_str().bold());

                    let mut stderr = io::stderr();
                    eprint!("\r");
                    let _ = stderr.execute(terminal::Clear(terminal::ClearType::UntilNewLine));
                    let _ = stderr.execute(cursor::Hide);
                    eprint!(
                        "{} {} {}",
                        symbol_colored,
                        frame.to_string().cyan(),
                        msg_bold
                    );
                    let _ = stderr.flush();

                    i += 1;
                    thread::sleep(Duration::from_millis(80));
                }
            });
            (Some(tx), Some(handle))
        } else {
            (None, None)
        };

        let status = Self {
            message: msg,
            symbol: sym,
            start: Instant::now(),
            is_tty,
            active: true,
            tx,
            thread,
        };
        if !is_tty {
            status.print_running(false);
        }
        status
    }

    /// Update the status message
    ///
    /// Changes the text displayed next to the symbol.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::tools::ui::Status;
    /// let mut status = Status::start("Running Phase 1");
    /// // ... work ...
    /// status.update("Running Phase 2");
    /// status.success();
    /// ```
    pub fn update(&mut self, message: impl Into<String>) {
        if !self.active {
            return;
        }
        self.message = message.into();
        if self.is_tty {
            if let Some(tx) = &self.tx {
                let _ = tx.send((false, Some(self.message.clone())));
            }
        } else {
            self.print_running(true);
        }
    }

    fn stop_thread(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send((true, None));
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }

    /// Mark the operation as complete success
    ///
    /// Prints a green checkmark and the time elapsed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::tools::ui::Status;
    /// let status = Status::start("Connecting");
    /// // ... work ...
    /// status.success(); // Prints "✓ Connecting (0.01s)"
    /// ```
    pub fn success(mut self) {
        if !self.active {
            return;
        }
        self.stop_thread();

        let duration = self.start.elapsed();
        let time_str = format!("({:.2?})", duration).dim();
        let msg = format!("{} {}", self.message.as_str().bold(), time_str);

        self.print_done("✓".green(), &msg);
        self.active = false;
    }

    /// Mark the operation as failed
    ///
    /// Prints a red cross, the original message, and the error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::tools::ui::Status;
    /// let status = Status::start("Downloading");
    /// // ... fail ...
    /// status.error("Connection Refused"); // Prints "✕ Downloading" and then the error
    /// ```
    pub fn error(mut self, err: impl std::fmt::Display) {
        if !self.active {
            return;
        }
        self.stop_thread();

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
            // For TTY, the thread handles the printing. But we might call this before thread starts.
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
        self.stop_thread();
        if self.active && self.is_tty {
            let _ = io::stderr().execute(cursor::Show);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
