use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    TypeChar(char),
    Backspace,
    Restart,
    Submit,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Settings,
    History,
    Quit,
    Ignore,
}

pub struct Keybindings;

impl Keybindings {
    pub fn load() -> std::io::Result<Self> {
        Ok(Self)
    }

    pub fn handle(&self, key: &KeyEvent) -> Action {
        match key.code {
            KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' => {
                Action::Quit
            }
            KeyCode::Char(_) if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Ignore,
            KeyCode::Char(c) => Action::TypeChar(c),
            KeyCode::Backspace => Action::Backspace,
            KeyCode::Tab => Action::Restart,
            KeyCode::Enter => Action::Submit,
            KeyCode::Up => Action::MoveUp,
            KeyCode::Down => Action::MoveDown,
            KeyCode::Left => Action::MoveLeft,
            KeyCode::Right => Action::MoveRight,
            KeyCode::F(2) => Action::Settings,
            KeyCode::F(3) => Action::History,
            KeyCode::Esc => Action::Quit,
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

    #[test]
    fn arrows_map_to_menu_actions() {
        let binds = Keybindings::load().unwrap();
        assert_eq!(binds.handle(&event(KeyCode::Up)), Action::MoveUp);
        assert_eq!(binds.handle(&event(KeyCode::Down)), Action::MoveDown);
        assert_eq!(binds.handle(&event(KeyCode::Left)), Action::MoveLeft);
        assert_eq!(binds.handle(&event(KeyCode::Right)), Action::MoveRight);
    }

    #[test]
    fn f2_opens_settings() {
        let binds = Keybindings::load().unwrap();
        assert_eq!(binds.handle(&event(KeyCode::F(2))), Action::Settings);
    }

    #[test]
    fn f3_opens_history() {
        let binds = Keybindings::load().unwrap();
        assert_eq!(binds.handle(&event(KeyCode::F(3))), Action::History);
    }

    #[test]
    fn common_actions_unchanged() {
        let binds = Keybindings::load().unwrap();
        assert_eq!(binds.handle(&event(KeyCode::Char('a'))), Action::TypeChar('a'));
        assert_eq!(binds.handle(&event(KeyCode::Enter)), Action::Submit);
        assert_eq!(binds.handle(&event(KeyCode::Tab)), Action::Restart);
        assert_eq!(binds.handle(&event(KeyCode::Esc)), Action::Quit);
    }
}