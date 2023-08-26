use crate::inputs::key::Key;

use strum::IntoEnumIterator;
use strum_macros::{Display as StrumDisplay, EnumIter};

/// We define all available action
#[derive(Debug, Clone, Copy, Eq, PartialEq, StrumDisplay, EnumIter)]
pub enum Action {
    Quit,
    Sleep,
}

impl Action {
    /// List of key associated to action
    pub fn keys(&self) -> &[Key] {
        match self {
            Action::Quit => &[Key::Ctrl('c'), Key::Char('q')],
            Action::Sleep => &[Key::Char('s')],
        }
    }
}

/// The application should have some contextual actions.
#[derive(Default, Debug, Clone)]
pub struct Actions(Vec<Action>);

impl Actions {
    /// Given a key, find the corresponding action
    pub fn find(&self, key: Key) -> Option<Action> {
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

fn check_unicity(actions: &[Action]) -> Result<(), String> {
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
                key,
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
        check_unicity(&actions)
            .map_err(|e| format!("Invalid action unicity{}", e))
            .unwrap();
        Self(actions)
    }
}
