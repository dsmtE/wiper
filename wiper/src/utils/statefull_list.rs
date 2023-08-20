use ratatui::widgets::ListState;

#[derive(Clone)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn set_items(&mut self, items: Vec<T>) {
        self.state.select((!items.is_empty()).then_some(0));
        self.items = items;
    }

    pub fn next(&mut self) {
        if self.items.is_empty() { return; }
        self.state.select(
            self.state.selected()
            .map(|i| if i >= self.items.len() - 1 { 0 } else { i + 1 })
        );
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() { return; }
        self.state.select(
            self.state.selected()
            .map(|i| if i <= 0 { self.items.len() - 1 } else { i - 1 })
        );
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

}

impl<T> Default for StatefulList<T> {
    fn default() -> Self {
        Self {
            state: ListState::default(),
            items: Vec::new(),
        }
    }
}
