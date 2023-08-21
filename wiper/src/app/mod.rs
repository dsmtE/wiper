use log::{debug, warn};
use tui_textarea::TextArea;
use std::collections::HashSet;
use std::path::PathBuf;
use crate::utils::{statefull_list::StatefulList, key_display::KeyEventWrapper, focusable_text_area::FocusableTextArea};

use self::actions::Actions;
use crate::app::actions::Action;
use crossterm::event::KeyEvent;
use ratatui::widgets::ScrollbarState;
use crate::utils::walker::{get_dir_list_from_path, count_and_size, delete_entries};

pub mod actions;
pub mod ui;

#[derive(clap::Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[arg(help("root Path to search"), value_hint = clap::ValueHint::DirPath)]
    pub root_path: Option<PathBuf>,
    #[arg(help("regex filter"), long, default_value = "^node_modules$")]
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
    pub path_text_area: FocusableTextArea<'static>,
    pub filter_text_area: FocusableTextArea<'static>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            regex_filter: "^node_modules$".to_string(),
            entries: StatefulList::default(),
            entries_size: vec![],
            selected_entries_idx: HashSet::new(),
            path_text_area: FocusableTextArea::default(),
            filter_text_area: FocusableTextArea::default()
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

        let path = args.root_path.clone().unwrap_or_else(|| PathBuf::from("."));
        let regex_filter = args.regex_filter.clone();
        let state = AppState {
            path: path.clone(),
            regex_filter: regex_filter.clone(),
            path_text_area: FocusableTextArea::new(
                [path.to_str().unwrap()],
                "Relative Path (Active - Esc to unfocus)",
                "Relative Path (Inactive - p to focus)"),
            filter_text_area: FocusableTextArea::new(
                ["^node_modules$"],
                "Filter (Active - Esc to unfocus)",
                "Filter (Inactive - f to focus)"),
            ..Default::default()
        };
        
        let mut app = Self {
            actions: Actions::from_iter([
                Action::Quit,
                Action::DeleteSelectedEntries,
                Action::ToggleCurrent,
                Action::Up,
                Action::Down,
                Action::EditPath,
                Action::EditFilter,
                Action::UnfocusTextArea,
            ]),
            state,
        };

        app.scan_dir_update();

        app
    }
    /// Handle a user action
    pub fn key_event(&mut self, key_event: KeyEvent) -> AppReturn {
        if key_event.kind != crossterm::event::KeyEventKind::Press {
            warn!("Key event is not a press event");
            return AppReturn::Continue;
        }

        let optional_action = self.actions.find(key_event);

        if self.state.path_text_area.focused || self.state.filter_text_area.focused {
            if let Some(Action::UnfocusTextArea) = optional_action {

                // update path state value from text area 
                if self.state.path_text_area.focused {
                    self.state.path = PathBuf::from(self.state.path_text_area.lines()[0].clone());
                    self.state.path_text_area.set_focus(false);
                } else {
                    self.state.regex_filter = self.state.filter_text_area.lines()[0].clone();
                    self.state.filter_text_area.set_focus(false);
                }
                self.scan_dir_update();

                return AppReturn::Continue;
            }
            
            let input = tui_textarea::Input::from(key_event);
            self.state.path_text_area.input(input.clone());
            self.state.filter_text_area.input(input);

            return AppReturn::Continue;
        }

        if optional_action.is_none() {
            warn!("No action associated to {}", KeyEventWrapper(&key_event));
            return AppReturn::Continue;
        }

        let action = optional_action.unwrap();
        debug!("Run action [{:?}]", action);

        match action {
            Action::DeleteSelectedEntries => {
                let state = self.state();
                let entries_to_delete = state.selected_entries_idx
                    .iter()
                    .map(|idx| state.entries.items[*idx].clone())
                    .collect::<Vec<_>>();

                if let Err(e) = delete_entries(&entries_to_delete) {
                    warn!("Error while deleting entries: {}", e);
                }

                self.scan_dir_update();
            },
            Action::ToggleCurrent => {
                let state = self.state_mut();
                let current_idx = state.entries.state.selected();
                if let Some(idx) = current_idx {
                    if state.selected_entries_idx.contains(&idx) {
                        state.selected_entries_idx.remove(&idx);
                    } else {
                        state.selected_entries_idx.insert(idx);
                    }
                }
            },
            Action::Up => {
                let state = self.state_mut();
                state.entries.previous();
            },
            Action::Down => {
                let state = self.state_mut();
                state.entries.next();
            },
            Action::Quit => {
                return AppReturn::Exit;
            },
            Action::EditPath => {
                self.state.path_text_area.set_focus(true);
            },
            Action::EditFilter => {
                self.state.filter_text_area.set_focus(true);
            },
            // Should not happen because we check if we are focused before
            Action::UnfocusTextArea => {
                self.state.path_text_area.set_focus(false);
                self.state.filter_text_area.set_focus(false);
            },
        }
        AppReturn::Continue
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
