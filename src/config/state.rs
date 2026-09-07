//! Small persistent UI state (first-run hints, etc.). Kept separate from the
//! user-editable `config.toml` because it is written by the app itself.

use std::path::PathBuf;

/// App-maintained state that survives restarts but is not user-editable.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiState {
    /// True until the app has welcomed the user once.
    #[serde(default = "default_true")]
    pub first_run: bool,
    /// How many times the user has dismissed the first-run overlay.
    #[serde(default)]
    pub first_run_acknowledged: u32,
}

fn default_true() -> bool {
    true
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            first_run: true,
            first_run_acknowledged: 0,
        }
    }
}

impl UiState {
    /// Path to the per-user state file.
    pub fn file_path() -> Option<PathBuf> {
        let dirs = directories::ProjectDirs::from("com", "dytype", "dytype")?;
        Some(dirs.config_dir().join("state.toml"))
    }

    pub fn load() -> Self {
        Self::load_from(Self::file_path())
    }

    fn load_from(path: Option<PathBuf>) -> Self {
        let Some(path) = path else {
            return Self::default();
        };
        std::fs::read_to_string(path)
            .ok()
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        self.save_to(Self::file_path());
    }

    fn save_to(&self, path: Option<PathBuf>) {
        let Some(path) = path else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = toml::to_string(self) {
            let _ = std::fs::write(path, content);
        }
    }

    /// Records that the first-run hint has been seen, persisting the change.
    pub fn acknowledge_first_run(&mut self) {
        self.first_run = false;
        self.first_run_acknowledged += 1;
        self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_state_path(tag: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("dytype-state-test-{}-{tag}", std::process::id()))
            .join("state.toml")
    }

    #[test]
    fn default_is_first_run() {
        assert!(UiState::default().first_run);
        assert_eq!(UiState::default().first_run_acknowledged, 0);
    }

    #[test]
    fn missing_state_file_is_first_run() {
        let path = temp_state_path("missing");
        let _ = std::fs::remove_file(&path);
        assert!(UiState::load_from(Some(path)).first_run);
    }

    #[test]
    fn acknowledge_clears_first_run_and_persists() {
        let path = temp_state_path("ack");
        let state = UiState::default();
        state.save_to(Some(path.clone()));
        // First-run round-trips until acknowledged.
        assert!(UiState::load_from(Some(path.clone())).first_run);

        let mut loaded = UiState::load_from(Some(path.clone()));
        loaded.acknowledge_first_run_for(&path);
        let stored = UiState::load_from(Some(path));
        assert!(!stored.first_run);
        assert_eq!(stored.first_run_acknowledged, 1);
    }

    #[test]
    fn missing_dir_creates_on_save() {
        let path = temp_state_path("dir");
        let state = UiState {
            first_run: false,
            ..UiState::default()
        };
        state.save_to(Some(path.clone()));
        assert!(path.exists());
        let stored = UiState::load_from(Some(path));
        assert!(!stored.first_run);
    }

    impl UiState {
        fn acknowledge_first_run_for(&mut self, path: &std::path::Path) {
            self.first_run = false;
            self.first_run_acknowledged += 1;
            self.save_to(Some(path.to_path_buf()));
        }
    }
}
