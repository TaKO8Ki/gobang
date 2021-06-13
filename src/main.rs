mod app;
mod ui;

use crate::app::App;
use crate::app::InputMode;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{
    error::Error,
    io::stdout,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, widgets::TableState, Terminal};

enum Event<I> {
    Input(I),
    Tick,
}

pub struct StatefulTable<'a> {
    state: TableState,
    items: Vec<Vec<&'a str>>,
}

impl<'a> StatefulTable<'a> {
    fn new() -> StatefulTable<'a> {
        StatefulTable {
            state: TableState::default(),
            items: vec![
                vec!["Row11", "Row12", "Row13", "Row14", "Row15", "Row16"],
                vec!["Row11", "Row12", "Row13", "Row13", "Row13", "Row13"],
            ],
        }
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
}

/// Crossterm demo
#[derive(Debug)]
struct Cli {
    /// time in ms between two ticks.
    tick_rate: u64,
    /// whether unicode symbols are used to improve the overall look of the app
    enhanced_graphics: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli {
        tick_rate: 250,
        enhanced_graphics: true,
    };

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();

    let tick_rate = Duration::from_millis(cli.tick_rate);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    use sqlx::mysql::{MySqlPool, MySqlRow};
    use sqlx::Row as _;

    let mut app = App::new("Crossterm Demo", cli.enhanced_graphics);

    let pool = MySqlPool::connect("mysql://root:@localhost:3306/hoge").await?;
    let mut rows = sqlx::query("SELECT * FROM user").fetch(&pool);
    let mut tables = sqlx::query("show tables")
        .fetch_all(&pool)
        .await?
        .iter()
        .map(|table| table.get(0))
        .collect::<Vec<String>>();
    app.tables = tables;

    terminal.clear()?;
    let mut table = StatefulTable::new();

    loop {
        terminal.draw(|f| ui::draw(f, &mut app, &mut table).unwrap())?;
        match rx.recv()? {
            Event::Input(event) => match app.input_mode {
                InputMode::Normal => match event.code {
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        execute!(
                            terminal.backend_mut(),
                            LeaveAlternateScreen,
                            DisableMouseCapture
                        )?;
                        terminal.show_cursor()?;
                        break;
                    }
                    KeyCode::Up => table.previous(),
                    KeyCode::Down => table.next(),
                    _ => {}
                },
                InputMode::Editing => match event.code {
                    KeyCode::Enter => {
                        app.messages.push(vec![app.input.drain(..).collect()]);
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            },
            Event::Tick => (),
        }
    }

    Ok(())
}
