use crate::app::InputMode;
use crate::app::{App, FocusBlock};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub fn draw<B: Backend>(f: &mut Frame<'_, B>, app: &mut App) -> anyhow::Result<()> {
    if let FocusBlock::ConnectionList = app.focus_type {
        let percent_x = 60;
        let percent_y = 50;
        let conns = &app.user_config.as_ref().unwrap().conn;
        let connections: Vec<ListItem> = conns
            .iter()
            .map(|i| {
                ListItem::new(vec![Spans::from(Span::raw(i.database_url()))])
                    .style(Style::default().fg(Color::White))
            })
            .collect();
        let tasks = List::new(connections)
            .block(Block::default().borders(Borders::ALL).title("Connections"))
            .highlight_style(Style::default().fg(Color::Green))
            .style(match app.focus_type {
                FocusBlock::ConnectionList => Style::default().fg(Color::Green),
                _ => Style::default(),
            });
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_y) / 2),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage((100 - percent_y) / 2),
                ]
                .as_ref(),
            )
            .split(f.size());

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1];
        f.render_widget(Clear, area);
        f.render_stateful_widget(tasks, area, &mut app.selected_connection);
        return Ok(());
    }

    let main_chunks = Layout::default()
        .margin(2)
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(15), Constraint::Percentage(85)])
        .split(f.size());

    let left_chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(9),
                Constraint::Min(8),
                Constraint::Length(7),
            ]
            .as_ref(),
        )
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
            FocusBlock::DabataseList(false) => Style::default().fg(Color::Magenta),
            FocusBlock::DabataseList(true) => Style::default().fg(Color::Green),
            _ => Style::default(),
        });
    f.render_stateful_widget(tasks, left_chunks[0], &mut app.selected_database);

    let databases = app.databases.clone();
    let tables: Vec<ListItem> = databases[app.selected_database.selected().unwrap_or(0)]
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
            FocusBlock::TableList(false) => Style::default().fg(Color::Magenta),
            FocusBlock::TableList(true) => Style::default().fg(Color::Green),
            _ => Style::default(),
        });
    f.render_stateful_widget(tasks, left_chunks[1], &mut app.selected_table);

    let info: Vec<ListItem> = vec![
        format!(
            "created: {}",
            app.selected_table().unwrap().create_time.to_string()
        ),
        // format!(
        //     "updated: {}",
        //     app.selected_table().unwrap().update_time.to_string()
        // ),
        format!("rows: {}", app.record_table.rows.len()),
    ]
    .iter()
    .map(|i| {
        ListItem::new(vec![Spans::from(Span::raw(i.to_string()))])
            .style(Style::default().fg(Color::White))
    })
    .collect();
    let tasks = List::new(info)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Green));
    f.render_widget(tasks, left_chunks[2]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(5)].as_ref())
        .split(main_chunks[1]);

    let query = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Query"));
    f.render_widget(query, right_chunks[0]);
    match app.input_mode {
        InputMode::Normal => (),
        InputMode::Editing => f.set_cursor(
            right_chunks[0].x + app.input.width() as u16 + 1,
            right_chunks[0].y + 1,
        ),
    }

    let header_cells = app.record_table.headers[app.record_table.column_index..]
        .iter()
        .map(|h| Cell::from(h.to_string()).style(Style::default().fg(Color::White)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    let rows = app.record_table.rows.iter().map(|item| {
        let height = item[app.record_table.column_index..]
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item[app.record_table.column_index..]
            .iter()
            .map(|c| Cell::from(c.to_string()).style(Style::default().fg(Color::White)));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    let widths = (0..10)
        .map(|_| Constraint::Percentage(10))
        .collect::<Vec<Constraint>>();
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Records"))
        .highlight_style(Style::default().fg(Color::Green))
        .style(match app.focus_type {
            FocusBlock::RecordTable(false) => Style::default().fg(Color::Magenta),
            FocusBlock::RecordTable(true) => Style::default().fg(Color::Green),
            _ => Style::default(),
        })
        .widths(&widths);
    f.render_stateful_widget(t, right_chunks[1], &mut app.record_table.state);

    Ok(())
}
