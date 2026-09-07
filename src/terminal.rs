use std::io;

use crossterm::execute;
use crossterm::terminal::disable_raw_mode;

/// Restores the terminal to its pre-app state on any exit path, including an
/// unexpected panic. Holding a `TerminalGuard` ensures raw mode is disabled
/// and the alternate screen is left even if the code between acquisition and
/// the guard's `Drop` unwinds. This is the single point that guarantees a
/// crashed application never leaves the user's terminal unusable.
pub struct TerminalGuard {
    restored: bool,
}

impl TerminalGuard {
    /// Enters the alternate screen, raw mode and mouse capture, wrapping them
    /// in a guard.
    ///
    /// Returns `Err` if the terminal cannot be initialized, without leaving a
    /// half-configured state behind.
    pub fn enter() -> io::Result<Self> {
        use crossterm::event::EnableMouseCapture;
        use crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        let result = execute!(stdout, EnterAlternateScreen, EnableMouseCapture);
        if let Err(error) = result {
            let _ = disable_raw_mode();
            return Err(error);
        }
        Ok(Self { restored: false })
    }

    pub fn restore(&mut self) {
        if self.restored {
            return;
        }
        self.restored = true;
        use crossterm::cursor::Show;
        use crossterm::event::DisableMouseCapture;
        use crossterm::terminal::LeaveAlternateScreen;

        let mut stdout = io::stdout();
        let _ = execute!(stdout, Show, LeaveAlternateScreen, DisableMouseCapture);
        let _ = disable_raw_mode();
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        self.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_is_idempotent() {
        // `restore` may only run once; calling it repeatedly is a no-op and
        // must not panic, so double-drop style misuse stays safe.
        let mut guard = TerminalGuard { restored: false };
        let _ = &mut guard;
        guard.restore();
        guard.restore();
        assert!(guard.restored);
    }

    #[test]
    fn restored_flag_is_initially_false() {
        let guard = TerminalGuard { restored: false };
        assert!(!guard.restored);
    }
}
