use log::{debug, error, warn};
use std::collections::HashSet;
use std::path::PathBuf;
use crate::utils::statefull_list::StatefulList;

use self::actions::Actions;
use crate::app::actions::Action;
use crate::inputs::key::Key;
use crate::io::IoEvent;
use crate::utils::walker::{get_dir_list_from_path, is_node_modules, count_and_size};

pub mod actions;
pub mod ui;

#[derive(clap::Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[arg(help("root Path to search"), value_hint = clap::ValueHint::DirPath)]
    pub root_path: Option<PathBuf>,
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
    pub entries: StatefulList<(walkdir::DirEntry, u64)>,
    pub selected_entries_idx: HashSet<usize>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            entries: StatefulList::default(),
            selected_entries_idx: HashSet::new(),
        }
    }
}

/// The main application, containing the state
pub struct App {
    /// We could dispatch an IO event
    io_tx: tokio::sync::mpsc::Sender<IoEvent>,
    /// Contextual actions
    actions: Actions,
    /// State
    is_loading: bool,
    state: AppState,
}

impl App {
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> Self {
        Self {
            io_tx,
            actions: Actions::from_iter([Action::Quit]),
            is_loading: false,
            state: AppState::default(),
        }
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
                        .map(|idx| state.entries.items()[*idx].0.clone())
                        .collect::<Vec<_>>();
                    // Delete is an I/O action, we dispatch on the IO channel that's run on another thread
                    self.dispatch(IoEvent::DeleteEntries(entries_to_delete)).await;
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

    pub fn initialize_from_args(&mut self, args: &Arguments) {
        self.actions = Actions::from_iter([Action::Quit, Action::DeleteSelectedEntries, Action::ToggleCurrent, Action::Up, Action::Down]);
        if let Some(root_path) = &args.root_path {
            self.state.path = root_path.clone();
        }
        self.scan_dir_update();

    }

    pub fn scan_dir_update(&mut self) {
        let state = self.state_mut();
        state.entries.set_items(get_dir_list_from_path(&state.path, &is_node_modules).map(|e| {
            let (_, size) = count_and_size(e.path());
            (e, size)
        }
        ).collect::<Vec<_>>());
        state.selected_entries_idx.clear();
    }

    /// We could update the app or dispatch event on tick
    pub async fn update_on_tick(&mut self) -> AppReturn {
        AppReturn::Continue
    }

    /// Send a network event to the IO thread
    pub async fn dispatch<'a>(&mut self, action: IoEvent) {
        // `is_loading` will be set to false again after the async action has finished in io/handler.rs
        self.is_loading = true;
        if let Err(e) = self.io_tx.send(action).await {
            self.is_loading = false;
            error!("Error from dispatch {}", e);
        };
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

    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    pub fn loaded(&mut self) {
        self.is_loading = false;
    }

}
