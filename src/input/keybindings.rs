use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::command::{Command, Keymap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    TypeChar(char),
    Backspace,
    Command(Command),
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Submit,
    Ignore,
}

/// Maps a raw key event onto either a high-level `Command` (via the
/// configurable keymap) or one of the low-level screen inputs.
pub struct Keybindings {
    keymap: Keymap,
}

impl Keybindings {
    pub fn new(keymap: Keymap) -> Self {
        Self { keymap }
    }

    pub fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    pub fn handle(&self, key: &KeyEvent) -> Action {
        if let Some(command) = self.keymap.resolve(key) {
            return Action::Command(command);
        }
        match key.code {
            KeyCode::Char(_) if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Ignore,
            KeyCode::Char(c) => Action::TypeChar(c),
            KeyCode::Backspace => Action::Backspace,
            // Some Windows consoles report the backspace key as Delete in raw
            // mode; treat it as a backspace so deletion keeps working there.
            KeyCode::Delete => Action::Backspace,
            KeyCode::Enter => Action::Submit,
            KeyCode::Up => Action::MoveUp,
            KeyCode::Down => Action::MoveDown,
            KeyCode::Left => Action::MoveLeft,
            KeyCode::Right => Action::MoveRight,
            _ => Action::Ignore,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    fn event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn binds() -> Keybindings {
        Keybindings::new(Keymap::default())
    }

    #[test]
    fn arrows_map_to_menu_actions() {
        let binds = binds();
        assert_eq!(binds.handle(&event(KeyCode::Up)), Action::MoveUp);
        assert_eq!(binds.handle(&event(KeyCode::Down)), Action::MoveDown);
        assert_eq!(binds.handle(&event(KeyCode::Left)), Action::MoveLeft);
        assert_eq!(binds.handle(&event(KeyCode::Right)), Action::MoveRight);
    }

    #[test]
    fn f2_opens_settings() {
        let binds = binds();
        assert_eq!(
            binds.handle(&event(KeyCode::F(2))),
            Action::Command(Command::OpenSettings)
        );
    }

    #[test]
    fn f3_opens_history() {
        let binds = binds();
        assert_eq!(
            binds.handle(&event(KeyCode::F(3))),
            Action::Command(Command::OpenHistory)
        );
    }

    #[test]
    fn common_actions_unchanged() {
        let binds = binds();
        assert_eq!(binds.handle(&event(KeyCode::Char('a'))), Action::TypeChar('a'));
        assert_eq!(binds.handle(&event(KeyCode::Enter)), Action::Submit);
        assert_eq!(
            binds.handle(&event(KeyCode::Tab)),
            Action::Command(Command::Restart)
        );
        assert_eq!(
            binds.handle(&event(KeyCode::Esc)),
            Action::Command(Command::Quit)
        );
    }

    #[test]
    fn ctrl_combos_resolve_to_commands() {
        let binds = binds();
        let ctrl = |c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL);
        assert_eq!(binds.handle(&ctrl('c')), Action::Command(Command::Quit));
        assert_eq!(binds.handle(&ctrl('r')), Action::Command(Command::Restart));
        assert_eq!(binds.handle(&ctrl('p')), Action::Command(Command::Pause));
        assert_eq!(binds.handle(&ctrl('t')), Action::Command(Command::ToggleStats));
        assert_eq!(binds.handle(&ctrl('x')), Action::Ignore);
    }

    #[test]
    fn delete_is_treated_as_backspace_for_windows_consoles() {
        let binds = binds();
        assert_eq!(binds.handle(&event(KeyCode::Delete)), Action::Backspace);
    }
}