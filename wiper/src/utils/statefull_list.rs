use tui::widgets::ListState;

#[derive(Clone)]
pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn from_items(items: Vec<T>) -> StatefulList<T> {
        let mut state = ListState::default();
        state.select(Some(0));
        StatefulList {
            state,
            items,
        }
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.state = ListState::default();
    }

    pub fn items(&self) -> &Vec<T> {
        &self.items
    }

    pub fn state(&self) -> &ListState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut ListState {
        &mut self.state
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

}

impl<T> Default for StatefulList<T> {
    fn default() -> Self {
        Self::from_items(Vec::new())
    }
}