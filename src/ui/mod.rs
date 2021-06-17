use crate::app::InputMode;
use crate::app::{App, FocusType};
use crate::StatefulTable;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans, Text},
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, Cell, Chart, Dataset, Gauge, LineGauge, List, ListItem,
        Paragraph, Row, Sparkline, Table, Tabs, Wrap,
    },
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub fn draw<B: Backend>(
    f: &mut Frame<'_, B>,
    app: &mut App,
    table: &mut StatefulTable,
) -> anyhow::Result<()> {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(15), Constraint::Percentage(85)])
        .direction(Direction::Horizontal)
        .split(f.size());

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(main_chunks[0]);
    let databases: Vec<ListItem> = app
        .databases
        .iter()
        .map(|i| {
            ListItem::new(vec![Spans::from(Span::raw(&i.name))])
                .style(Style::default().fg(Color::White))
        })
        .collect();
    let tasks = List::new(databases)
        .block(Block::default().borders(Borders::ALL).title("Databases"))
        .highlight_style(Style::default().fg(Color::Green))
        .style(match app.focus_type {
            FocusType::Dabatases(false) => Style::default().fg(Color::Magenta),
            FocusType::Dabatases(true) => Style::default().fg(Color::Green),
            _ => Style::default(),
        });
    f.render_stateful_widget(tasks, left_chunks[0], &mut app.selected_database);

    let databases = app.databases.clone();
    let tables: Vec<ListItem> = databases[match app.selected_database.selected() {
        Some(index) => index,
        None => 0,
    }]
    .tables
    .iter()
    .map(|i| {
        ListItem::new(vec![Spans::from(Span::raw(&i.name))])
            .style(Style::default().fg(Color::White))
    })
    .collect();
    let tasks = List::new(tables)
        .block(Block::default().borders(Borders::ALL).title("Tables"))
        .highlight_style(Style::default().fg(Color::Green))
        .style(match app.focus_type {
            FocusType::Tables(false) => Style::default().fg(Color::Magenta),
            FocusType::Tables(true) => Style::default().fg(Color::Green),
            _ => Style::default(),
        });
    f.render_stateful_widget(
        tasks,
        left_chunks[1],
        &mut app.databases[app.selected_database.selected().unwrap_or(0)].selected_table,
    );

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(main_chunks[1]);

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, right_chunks[0]);

    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, right_chunks[1]);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                right_chunks[1].x + app.input.width() as u16 + 1,
                // Move one line down, from the border to the input line
                right_chunks[1].y + 1,
            )
        }
    }

    let header_cells = table
        .headers
        .iter()
        .map(|h| Cell::from(h.to_string()).style(Style::default().fg(Color::White)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    let rows = table.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item
            .iter()
            .map(|c| Cell::from(c.to_string()).style(Style::default().fg(Color::White)));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Records"))
        .highlight_style(Style::default().fg(Color::Green))
        .style(match app.focus_type {
            FocusType::Records(false) => Style::default().fg(Color::Magenta),
            FocusType::Records(true) => Style::default().fg(Color::Green),
            _ => Style::default(),
        })
        .widths(&[
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ]);
    f.render_stateful_widget(t, right_chunks[2], &mut table.state);

    Ok(())
}
