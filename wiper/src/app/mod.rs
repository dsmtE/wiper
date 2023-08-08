use log::{debug, warn, info};
use std::collections::HashSet;
use std::path::PathBuf;
use crate::utils::statefull_list::StatefulList;

use self::actions::Actions;
use crate::app::actions::Action;
use crate::inputs::key::Key;
use crate::utils::walker::{get_dir_list_from_path, count_and_size, delete_entries};

pub mod actions;
pub mod ui;

#[derive(clap::Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[arg(help("root Path to search"), value_hint = clap::ValueHint::DirPath)]
    pub root_path: Option<PathBuf>,
    #[arg(help("regex filter"), long, default_value = "node_modules")]
    pub regex_filter: String,
    #[arg(
        short,
        long,
        default_value_t = true,
        help("do not search on subfolders")
    )]
    pub prune: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

#[derive(Clone)]
pub struct AppState {
    pub path: PathBuf,
    pub regex_filter: String,
    pub entries: StatefulList<walkdir::DirEntry>,
    pub entries_size: Vec<u64>,
    pub selected_entries_idx: HashSet<usize>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            regex_filter: String::default(),
            entries: StatefulList::default(),
            entries_size: vec![],
            selected_entries_idx: HashSet::new(),
        }
    }
}

/// The main application, containing the state
pub struct App {
    /// Contextual actions
    actions: Actions,
    state: AppState,
}

impl App {
    pub fn new_from_args(args: &Arguments) -> Self {
        let mut app = Self {
            actions: Actions::from_iter([
                Action::Quit,
                Action::DeleteSelectedEntries,
                Action::ToggleCurrent,
                Action::Up,
                Action::Down
                ]),
            state: AppState {
                path: args.root_path.clone().unwrap_or_else(|| PathBuf::from(".")),
                regex_filter: args.regex_filter.clone(),
                ..Default::default()
            },
        };

        app.scan_dir_update();

        app
    }

    /// Handle a user action
    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        if let Some(action) = self.actions.find(key) {
            debug!("Run action [{:?}]", action);
            match action {
                Action::Quit => AppReturn::Exit,
                Action::DeleteSelectedEntries => {
                    let state = self.state();
                    let entries_to_delete = state.selected_entries_idx
                        .iter()
                        .map(|idx| state.entries.items()[*idx].clone())
                        .collect::<Vec<_>>();
 
                    if let Err(e) = delete_entries(&entries_to_delete) {
                        warn!("Error while deleting entries: {}", e);
                    }

                    self.scan_dir_update();

                    AppReturn::Continue
                },
                Action::ToggleCurrent => {
                    let state = self.state_mut();
                    let current_idx = state.entries.state().selected();
                    if let Some(idx) = current_idx {
                        if state.selected_entries_idx.contains(&idx) {
                            state.selected_entries_idx.remove(&idx);
                        } else {
                            state.selected_entries_idx.insert(idx);
                        }
                    }
                    AppReturn::Continue
                },
                Action::Up => {
                    let state = self.state_mut();
                    state.entries.previous();
                    AppReturn::Continue
                },
                Action::Down => {
                    let state = self.state_mut();
                    state.entries.next();
                    AppReturn::Continue
                },
            }
        } else {
            warn!("No action accociated to {}", key);
            AppReturn::Continue
        }
    }

    pub fn scan_dir_update(&mut self) {
        let state = self.state_mut();
        let regex_fiter = regex::Regex::new(&state.regex_filter).unwrap();

        let mut dir_entries = get_dir_list_from_path(
            &state.path,
            &|entry| {
                regex_fiter.is_match(entry.to_str().unwrap())
            }
            )
        .collect::<Vec<_>>();

        let mut entries_size = dir_entries
            .iter()
            .map(|entry| count_and_size(entry.path()).1)
            .collect::<Vec<_>>();

        // compute permutation sorted by size
        let mut permutation = (0..entries_size.len()).collect::<Vec<usize>>();
        permutation.sort_by(|a, b| entries_size[*b].cmp(&entries_size[*a]));

        // apply permutation
        dir_entries = permutation.iter().map(|&idx| dir_entries[idx].clone()).collect();
        entries_size = permutation.iter().map(|&idx| entries_size[idx]).collect();

        state.entries.set_items(dir_entries);
        state.entries_size = entries_size;
        state.selected_entries_idx.clear();
    }

    /// We could update the app or dispatch event on tick
    pub async fn update_on_tick(&mut self) -> AppReturn {
        AppReturn::Continue
    }

    pub fn actions(&self) -> &Actions {
        &self.actions
    }
    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

}
