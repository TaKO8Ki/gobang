use crate::app::{App, FocusBlock, Tab};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Tabs},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub fn draw<B: Backend>(f: &mut Frame<'_, B>, app: &mut App) -> anyhow::Result<()> {
    if let FocusBlock::ConnectionList = app.focus_block {
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
            .style(match app.focus_block {
                FocusBlock::ConnectionList => Style::default().fg(Color::Green),
                _ => Style::default().fg(Color::DarkGray),
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
        .style(match app.focus_block {
            FocusBlock::DabataseList(false) => Style::default(),
            FocusBlock::DabataseList(true) => Style::default().fg(Color::Green),
            _ => Style::default().fg(Color::DarkGray),
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
        .style(match app.focus_block {
            FocusBlock::TableList(false) => Style::default(),
            FocusBlock::TableList(true) => Style::default().fg(Color::Green),
            _ => Style::default().fg(Color::DarkGray),
        });
    f.render_stateful_widget(tasks, left_chunks[1], &mut app.selected_table);

    let table_status: Vec<ListItem> = app
        .table_status()
        .iter()
        .map(|i| {
            ListItem::new(vec![Spans::from(Span::raw(i.to_string()))])
                .style(Style::default().fg(Color::White))
        })
        .collect();
    let tasks = List::new(table_status)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Green));
    f.render_widget(tasks, left_chunks[2]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(5),
            ]
            .as_ref(),
        )
        .split(main_chunks[1]);

    let titles = Tab::names().iter().cloned().map(Spans::from).collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL))
        .select(app.selected_tab as usize)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Reset)
                .add_modifier(Modifier::UNDERLINED),
        );
    f.render_widget(tabs, right_chunks[0]);

    let query = Paragraph::new(app.input.as_ref())
        .style(match app.focus_block {
            FocusBlock::Query(true) => Style::default().fg(Color::Green),
            FocusBlock::Query(false) => Style::default(),
            _ => Style::default().fg(Color::DarkGray),
        })
        .block(Block::default().borders(Borders::ALL).title("Query"));
    f.render_widget(query, right_chunks[1]);
    if let FocusBlock::Query(true) = app.focus_block {
        f.set_cursor(
            right_chunks[1].x + app.input.width() as u16 + 1 - app.input_cursor_x,
            right_chunks[1].y + 1,
        )
    }
    match app.selected_tab {
        Tab::Records => draw_records_table(f, app, right_chunks[2])?,
        Tab::Structure => draw_structure_table(f, app, right_chunks[2])?,
    }
    if let Some(err) = app.error.clone() {
        draw_error_popup(f, err)?;
    }
    Ok(())
}

fn draw_structure_table<B: Backend>(
    f: &mut Frame<'_, B>,
    app: &mut App,
    layout_chunk: Rect,
) -> anyhow::Result<()> {
    let headers = app.structure_table.headers();
    let header_cells = headers
        .iter()
        .map(|h| Cell::from(h.to_string()).style(Style::default().fg(Color::White)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    let rows = app.structure_table.rows();
    let rows = rows.iter().map(|item| {
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
    let widths = (0..10)
        .map(|_| Constraint::Percentage(10))
        .collect::<Vec<Constraint>>();
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Structure"))
        .highlight_style(Style::default().fg(Color::Green))
        .style(match app.focus_block {
            FocusBlock::RecordTable(false) => Style::default(),
            FocusBlock::RecordTable(true) => Style::default().fg(Color::Green),
            _ => Style::default().fg(Color::DarkGray),
        })
        .widths(&widths);
    f.render_stateful_widget(t, layout_chunk, &mut app.structure_table.state);
    Ok(())
}

fn draw_records_table<B: Backend>(
    f: &mut Frame<'_, B>,
    app: &mut App,
    layout_chunk: Rect,
) -> anyhow::Result<()> {
    let headers = app.record_table.headers();
    let header_cells = headers
        .iter()
        .map(|h| Cell::from(h.to_string()).style(Style::default().fg(Color::White)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    let rows = app.record_table.rows();
    let rows = rows.iter().map(|item| {
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
    let widths = (0..10)
        .map(|_| Constraint::Percentage(10))
        .collect::<Vec<Constraint>>();
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Records"))
        .highlight_style(Style::default().fg(Color::Green))
        .style(match app.focus_block {
            FocusBlock::RecordTable(false) => Style::default(),
            FocusBlock::RecordTable(true) => Style::default().fg(Color::Green),
            _ => Style::default().fg(Color::DarkGray),
        })
        .widths(&widths);
    f.render_stateful_widget(t, layout_chunk, &mut app.record_table.state);
    Ok(())
}

fn draw_error_popup<B: Backend>(f: &mut Frame<'_, B>, error: String) -> anyhow::Result<()> {
    let percent_x = 60;
    let percent_y = 20;
    let error = Paragraph::new(error)
        .block(Block::default().title("Error").borders(Borders::ALL))
        .style(Style::default().fg(Color::Red));
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
    f.render_widget(error, area);
    Ok(())
}
