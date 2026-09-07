# Changelog

All notable changes to dytype are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- Code snippets mode (`TestKind::Code`): type real multi-line code snippets
  line by line, preserving indentation and layout. Built-in `rust`, `python`,
  and `javascript` snippets, with optional `assets/snippets/*.txt` overrides.
- Error-combination analysis: per-character stats now carry bigram/trigram
  context (`prev`/`prev2`), surfaced as weak sequences in the History Errors
  tab. Legacy three-part DB records still decode (backward compatible).
- First-run welcome hint on the config menu, dismissed when the first test
  starts and persisted in `state.toml`.
- User-authored themes loaded from `~/.config/dytype/themes/*.toml` (hex or
  named colors).
- Language packs: selectable word/quote lists per language, with the German
  pack shipped first and English always available.
- Mouse support in menus, settings, history, and the results screen
  (configurable via `display.mouse`).
- Help screen (`F1`) summarizing keybindings.
- Per-command keybinding overrides in `[keybindings.overrides]`.

### Changed
- History Errors tab now shows weak bigrams/trigrams alongside the existing
  per-character and duration breakdowns.
- Config menu gained a fifth mode (`Code`).

### Fixed
- Help screen previously panicked when a line's styled spans exceeded one
  line; screen rendering is now robust on short terminals.
- Terminal mouse capture is restored after a panic via a hook.

### Removed
- Nothing.

## [0.1.0] - initial

### Added
- Timed and per-word typing tests with real-time WPM, accuracy, and
  consistency.
- Persistent test history with summary, progression trends, per-character
  errors, and duration distribution.
- Configurable themes, keybindings, sounds, and cursor behavior.
- Mechanical, typewriter, soft, and retro sound packs.
