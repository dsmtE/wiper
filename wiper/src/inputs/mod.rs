use self::key::Key;

pub mod events;
pub mod key;

pub enum InputEvent {
    /// An input event occurred.
    Pressed(Key),
    Released(Key),
    Repeat(Key),
    /// An tick event occurred.
    Tick,
}
