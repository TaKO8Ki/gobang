use tui::widgets::TableState;

pub struct RecordTable {
    state: TableState,
    column_names: Vec<String>,
    records: Vec<Vec<String>>,
}

impl RecordTable {
    fn new() -> Self {
        Self {
            state: TableState::default(),
            column_names: vec![],
            records: vec![],
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.records.len() - 1 {
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
                    self.records.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
