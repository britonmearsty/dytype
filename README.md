# dytype

A terminal typing test application with a clean, configurable, animation-rich
TUI. The typing engine is the product; the TUI is the interface around it.

## Features

- Timed, per-word, quote, practice, and **code-snippet** typing tests
- Real-time WPM, accuracy, and consistency stats
- Test history with persistent storage, progression, achievements, and
  per-character / weak-sequence (bigram & trigram) error analysis
- Multiple themes plus user-authored TOML themes
- Language packs (English, German, and more under `assets/words/`)
- Mechanical, typewriter, soft, and retro key sounds
- Configurable keybindings, themes, and sounds
- Mouse support in menus and history

## Install

Prebuilt binaries for Linux, macOS (Intel & Apple Silicon), and Windows are
attached to each [release](https://github.com/britonmearsty/dytype/releases).
The one-line installer fetches the right one, verifies its checksum, and puts
it in `~/.local/bin`:

```sh
curl -fsSL https://raw.githubusercontent.com/britonmearsty/dytype/main/install.sh | sh
```

The installer prints what it downloads and where it installs; `sh install.sh --help`
lists its options (choose a directory, pin a version), and every download is
verified against SHA256SUMS attached to the release.

Or from source (needs a Rust toolchain):

```sh
cargo install --path .
dytype
```

Or run without installing:

```sh
cargo run
```

## Usage

Start a test, type the prompt, and finish to see your results. `Enter` on the
main menu restarts; in a test it submits early.

### Key map

| Keys | Action |
|------|--------|
| `F1` | Help |
| `F2` | Settings |
| `F3` | History |
| `F4` / `F5` | Previous / next test (history) |
| `Ctrl+C`, `Esc` | Quit |
| `Ctrl+R`, `Tab` | Restart |
| `Ctrl+P` | Pause / resume |
| `Ctrl+T` | Toggle live stats |
| `↑` / `↓` | Select row |
| `←` / `→` | Change value (menus) or switch history tab |
| `Enter` | Confirm / start |
| `Backspace` | Correct a mistake mid-test (if enabled) |

Every binding is remappable; see the Key bindings section below.

### Modes

- **Words** — type a fixed number of words.
- **Time** — type as much as you can in the chosen duration.
- **Quote** — type a full quote word-for-word.
- **Practice** — re-type your most-missed words.
- **Code** — type a real code snippet line by line, preserving indentation.

## Configuration

On first run dytype writes nothing until you change a setting; every value
has a sensible default. To configure, create a config file:

- **Linux/macOS:** `~/.config/dytype/config.toml`
- **Windows:** `%APPDATA%\dytype\config.toml`

A fully annotated example lives at [`dytype.example.toml`](dytype.example.toml);
you can copy it and edit to taste. Any omitted field falls back to its default
and unknown fields are ignored, so configs remain forward compatible.

### Key bindings

Override individual built-in bindings in the `[keybindings.overrides]` table.
Keys are written like `"ctrl+c"`, `"F1"`, `"tab"`, or `"esc"`:

```toml
[keybindings.overrides]
restart = "ctrl+k"
toggle_stats = "F8"
```

### Custom themes

Drop a TOML file into your themes directory to add a theme:

- **Linux/macOS:** `~/.config/dytype/themes/<name>.toml`
- **Windows:** `%APPDATA%\dytype\themes\<name>.toml`

Colors accept `#RRGGBB`, `#RGB`, or named CSS colors:

```toml
name = "gruvbox"
background = "#282828"
foreground = "#ebdbb2"
text = "#ebdbb2"
correct = "#b8bb26"
incorrect = "#fb4934"
cursor = "#fe8019"
muted = "#928374"
accent = "#83a598"
```

Then select it by setting `theme.name = "gruvbox"` in your config.

## Project Layout

```
src/
├── main.rs          # entry point
├── app.rs           # top-level application state and event loop
├── input/           # input handling and keybindings
├── typing/          # engine, words, snippets, cursor, test logic
├── stats/           # WPM, accuracy, consistency, history, analysis
├── ui/              # terminal UI screens and widgets
├── audio/           # key sound playback
├── config/          # settings and keybinding configuration
└── persistence/     # test history storage
assets/
├── words/           # `{language}-{difficulty}.txt` lists
├── quotes/          # `{language}.txt` quote lists
└── snippets/        # optional `{name}.txt` code snippets
```

### Assets

- **Words & quotes** — plain text files, one entry per line, loaded at startup
  and per selected language. Missing assets fall back to the embedded English
  pools, so tests always have material.
- **Snippets** — per-line code files used by Code mode. Files in
  `assets/snippets/` override the built-in `rust`, `python`, and `javascript`
  snippets.

## Development

```sh
cargo test            # unit + integration tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## License

MIT
