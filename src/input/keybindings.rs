use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    TypeChar(char),
    Backspace,
    Restart,
    Submit,
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
            KeyCode::Esc => Action::Quit,
            _ => Action::Ignore,
        }
    }
}