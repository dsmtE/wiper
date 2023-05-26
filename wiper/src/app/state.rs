use std::collections::HashSet;

use crate::utils::statefull_list::StatefulList;

#[derive(Clone)]
pub struct AppState {
    pub path: String,
    pub counter_tick: u64,
    pub entries: StatefulList<(walkdir::DirEntry, u64)>,
    pub selected_entries_idx: HashSet<usize>,
}

impl AppState {
    pub fn incr_tick(&mut self) {
            self.counter_tick += 1;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            path: String::new(),
            counter_tick: 0,
            entries: StatefulList::default(),
            selected_entries_idx: HashSet::new(),
        }
    }
}
