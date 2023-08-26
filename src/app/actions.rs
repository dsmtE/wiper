use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use strum::IntoEnumIterator;
use strum_macros::{Display as StrumDisplay, EnumIter};

use crate::utils::key_display::KeyEventWrapper;

/// We define all available action
#[derive(Debug, Clone, Copy, Eq, PartialEq, StrumDisplay, EnumIter)]
pub enum Action {
    Quit,
    ToggleCurrent,
    DeleteSelectedEntries,
    Up,
    Down,
    EditPath,
    EditFilter,
    UnfocusTextArea,
}

impl Action {
    /// List of key associated to action
    pub fn keys(&self) -> Vec<KeyEvent> {
        match self {
            Action::Quit => vec![KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL), KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)],
            Action::ToggleCurrent => vec![KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)],
            Action::DeleteSelectedEntries => vec![KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE)],
            Action::Up => vec![KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)],
            Action::Down => vec![KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)],
            Action::EditPath => vec![KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE)],
            Action::EditFilter => vec![KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE)],
            Action::UnfocusTextArea => vec![KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)],
        }
    }
}

/// The application should have some contextual actions.
#[derive(Default, Debug, Clone)]
pub struct Actions(Vec<Action>);

impl Actions {
    /// Given a key, find the corresponding action
    pub fn find(&self, key: KeyEvent) -> Option<Action> {
        Action::iter()
            .filter(|action| self.0.contains(action))
            .find(|action| action.keys().contains(&key))
    }

    pub fn slice(&self) -> &[Action] {
        self.0.as_slice()
    }
}

impl From<Vec<Action>> for Actions {
    fn from(actions: Vec<Action>) -> Self {
        Self(actions)
    }
}

fn vec_to_string<T: std::fmt::Display>(vec: &[T], separator: &'static str) -> String {
    vec.iter()
        .map(|item| item.to_string())
        .reduce(|mut a, b| {
            a.reserve(b.len() + separator.len());
            a.push_str(separator);
            a.push_str(&b);
            a
        })
        .unwrap()
}

fn check_action_conflicts(actions: &[Action]) -> Result<(), String> {
    let mut map = std::collections::HashMap::new();

    for action in actions {
        for key in action.keys().iter() {
            map.entry(*key).or_insert_with(Vec::new).push(*action);
        }
    }

    let errors = map
        .iter()
        .filter(|(_, actions)| actions.len() > 1) // at least two actions share same shortcut
        .map(|(key, actions)| {
            format!(
                "Conflict key {} with actions {}",
                KeyEventWrapper(key),
                vec_to_string(actions, ", ")
            )
        })
        .collect::<Vec<_>>();

    errors
        .is_empty()
        .then_some(())
        .ok_or_else(|| errors.join("; "))
}

impl FromIterator<Action> for Actions {
    fn from_iter<T: IntoIterator<Item = Action>>(iter: T) -> Self {
        let actions = iter.into_iter().collect::<Vec<_>>();
        check_action_conflicts(&actions)
            .map_err(|e|  format!("Error while creating actions: {}", e))
            .unwrap();
        Self(actions)
    }
}
