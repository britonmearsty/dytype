/// Sound-level events produced by the typing engine and consumed by the
/// audio manager. Keeping the event type separate from the audio backends
/// means the engine never touches playback details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundEvent {
    Keypress,
    Error,
    WordComplete,
    TestComplete,
}
