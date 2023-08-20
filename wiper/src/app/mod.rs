use log::{debug, warn};
use tui_textarea::TextArea;
use std::collections::HashSet;
use std::path::PathBuf;
use crate::utils::{statefull_list::StatefulList, key_display::KeyEventWrapper};

use self::actions::Actions;
use crate::app::actions::Action;
use crossterm::event::KeyEvent;
use ratatui::{
    style::{Style, Modifier, Color},
    widgets::{Block, Borders}
};
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
    pub focused_text_area: Option<u8>,
    pub path_text_area: TextArea<'static>,
    pub filter_text_area: TextArea<'static>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            regex_filter: "^node_modules$".to_string(),
            entries: StatefulList::default(),
            entries_size: vec![],
            selected_entries_idx: HashSet::new(),
            focused_text_area: None,
            path_text_area: TextArea::from(["."]),
            filter_text_area: TextArea::from(["^node_modules$"]),
        }
    }
}

fn activate_text_area(text_area: & mut TextArea, title: String)
    {
        text_area.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
        text_area.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        let b = text_area
            .block()
            .cloned()
            .unwrap_or_else(|| Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::LightGreen));
        text_area.set_block(b.title(title));
    }

fn inactivate_text_area(text_area: &mut TextArea, title: String) {
        text_area.set_cursor_line_style(Style::default());
        text_area.set_cursor_style(Style::default());
        let b = text_area
            .block()
            .cloned()
            .unwrap_or_else(|| Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray));
        text_area.set_block(b.title(title));
    }

impl AppState {
    fn activate_focused_text_area(&mut self) {

        if self.focused_text_area.is_none() {
            return;
        }

        let text_area = if self.focused_text_area.unwrap() == 0 {
            &self.path_text_area
        }else {
            &self.filter_text_area
        };
        if std::ptr::eq(&self.path_text_area, text_area) {
            activate_text_area(&mut self.path_text_area, "Relative Path (Active - Esc to unfocus)".to_string());
        }else {
            activate_text_area(&mut self.filter_text_area, "Filter (Inactive - f to focus)".to_string());
        }
        
    }

    fn inactivate_focused_text_area(&mut self) {

        if self.focused_text_area.is_none() {
            return;
        }

        let text_area = if self.focused_text_area.unwrap() == 0 {
            &self.path_text_area
        }else {
            &self.filter_text_area
        };

        if std::ptr::eq(&self.path_text_area, text_area) {
            inactivate_text_area(&mut self.path_text_area, "Relative Path (Inactive - p to focus)".to_string());
        }else {
            inactivate_text_area(&mut self.filter_text_area, "Filter (Inactive - f to focus)".to_string());
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
        let mut state = AppState {
            path: path.clone(),
            regex_filter: regex_filter.clone(),
            path_text_area : TextArea::from([path.to_str().unwrap()]),
            filter_text_area : TextArea::from([regex_filter]),
            ..Default::default()
        };
        
        inactivate_text_area(&mut state.path_text_area, "Relative Path (Inactive - p to focus)".to_string());
        inactivate_text_area(&mut state.filter_text_area, "Filter (Inactive - f to focus)".to_string());

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

        if let Some(n) = self.state.focused_text_area {
            if let Some(Action::UnfocusTextArea) = optional_action {

                // update path state value from text area 
                if n == 0 {
                    self.state.path = PathBuf::from(self.state.path_text_area.lines()[0].clone());
                } else {
                    self.state.regex_filter = self.state.filter_text_area.lines()[0].clone();
                }
                self.scan_dir_update();

                self.state.inactivate_focused_text_area();
                self.state.focused_text_area = None;

                return AppReturn::Continue;
            }
            
            let text_area = if self.state.focused_text_area.unwrap() == 0 {
                &mut self.state.path_text_area
            }else {
                &mut self.state.filter_text_area
            };

            text_area.input(tui_textarea::Input::from(key_event));
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
                    .map(|idx| state.entries.items()[*idx].clone())
                    .collect::<Vec<_>>();

                if let Err(e) = delete_entries(&entries_to_delete) {
                    warn!("Error while deleting entries: {}", e);
                }

                self.scan_dir_update();
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
                self.state.focused_text_area = Some(0);
                self.state.activate_focused_text_area();
            },
            Action::EditFilter => {
                self.state.focused_text_area = Some(1);
                self.state.activate_focused_text_area();
            },
            // Should not happen because we check if we are focused before
            Action::UnfocusTextArea => {
                self.state.focused_text_area = None;
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
