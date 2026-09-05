# dytype

A terminal typing test application.

## Features

- Timed and per-word typing tests
- Real-time WPM, accuracy, and consistency stats
- Test history with persistent storage
- Mechanical and typewriter key sounds
- Configurable keybindings and themes

## Usage

```sh
cargo run
```

## Project Layout

```
src/
├── main.rs          # entry point
├── app.rs           # top-level application state and event loop
├── input/           # input handling and keybindings
├── typing/          # typing engine, words, cursor, test logic
├── stats/           # WPM, accuracy, consistency, history
├── ui/              # terminal UI screens and widgets
├── audio/           # key sound playback
├── config/          # settings and keybinding configuration
└── persistence/     # test history storage
```

## License

MIT