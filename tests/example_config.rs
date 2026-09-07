use dytype::config::settings::Config;
use dytype::typing::generator::Difficulty;

#[test]
fn example_config_parses_and_keeps_defaults() {
    let content =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/dytype.example.toml"))
            .expect("example config file exists");
    let config: Config = toml::from_str(&content).expect("example config parses");
    assert_eq!(config.theme.name, "Default");
    assert_eq!(config.typing.difficulty, Difficulty::Normal);
    assert_eq!(config.typing.language, "English");
    assert!(config.sounds.enabled);
    assert_eq!(config.sounds.volume, 0.65);
    assert_eq!(config.display.fps, 60);
    assert!(config.display.mouse);
    assert!(config.display.cursor.blink);
    assert_eq!(
        config.display.cursor.style,
        dytype::config::settings::CursorStyle::Bar
    );
    assert!(config.keybindings.overrides.is_empty());
}
