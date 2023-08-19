use std::ops;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};

pub struct KeyEventWrapper<'a>(pub &'a KeyEvent);

impl<'a> ops::Deref for KeyEventWrapper<'a> {
    type Target = KeyEvent;

    fn deref(&self) -> &'a Self::Target {
        &self.0
    }
}

impl std::fmt::Display for KeyEventWrapper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let key_code_string = match self.code {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::F(n) => format!("F{}", n),
            _ => format!("{:?}", self.code),
        };

        let modifiers_str: &str = match self.modifiers {
            KeyModifiers::CONTROL => "ctrl",
            KeyModifiers::SHIFT => "shift",
            KeyModifiers::ALT => "alt",
            x if x == KeyModifiers::CONTROL | KeyModifiers::SHIFT => "ctrl+shift",
            x if x == KeyModifiers::CONTROL | KeyModifiers::ALT => "ctrl+alt",
            x if x == KeyModifiers::SHIFT | KeyModifiers::ALT => "shift+alt",
            x if x == KeyModifiers::CONTROL | KeyModifiers::SHIFT | KeyModifiers::ALT => "ctrl+alt+shift",
            KeyModifiers::NONE => "",
            _ => "unknown",
        };

        if self.modifiers == KeyModifiers::NONE {
            write!(f, "{}", key_code_string)
        } else {
            write!(f, "{}+{}", modifiers_str, key_code_string)
        }
    }
}