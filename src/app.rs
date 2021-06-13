pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    /// Current value of the input box
    pub input: String,
    /// Current input mode
    pub input_mode: InputMode,
    /// History of recorded messages
    pub messages: Vec<Vec<String>>,
    pub tables: Vec<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            tables: Vec::new(),
        }
    }
}

impl App {
    pub fn new(title: &str, enhanced_graphics: bool) -> App {
        Self::default()
    }
}
