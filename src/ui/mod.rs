use crate::app::{App, FocusBlock, Tab};
use crate::components::DrawableComponent as _;
use crate::event::Key;
use database_tree::MoveSelection;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};

pub mod scrollbar;
pub mod scrolllist;

pub fn draw<B: Backend>(f: &mut Frame<'_, B>, app: &mut App) -> anyhow::Result<()> {
    if let FocusBlock::ConnectionList = app.focus_block {
        let percent_x = 60;
        let percent_y = 50;
        let conns = &app.user_config.as_ref().unwrap().conn;
        let connections: Vec<ListItem> = conns
            .iter()
            .map(|i| {
                ListItem::new(vec![Spans::from(Span::raw(i.database_url()))])
                    .style(Style::default())
            })
            .collect();
        let tasks = List::new(connections)
            .block(Block::default().borders(Borders::ALL).title("Connections"))
            .highlight_style(Style::default().bg(Color::Blue))
            .style(match app.focus_block {
                FocusBlock::ConnectionList => Style::default(),
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
        .constraints([Constraint::Min(8), Constraint::Length(7)].as_ref())
        .split(main_chunks[0]);

    app.databases
        .draw(
            f,
            left_chunks[0],
            matches!(app.focus_block, FocusBlock::DabataseList),
        )
        .unwrap();

    let table_status: Vec<ListItem> = app
        .table_status()
        .iter()
        .map(|i| ListItem::new(vec![Spans::from(Span::raw(i.to_string()))]).style(Style::default()))
        .collect();
    let tasks = List::new(table_status).block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(tasks, left_chunks[1]);

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

    app.query.draw(
        f,
        right_chunks[1],
        matches!(app.focus_block, FocusBlock::Query),
    )?;

    match app.selected_tab {
        Tab::Records => app.record_table.draw(
            f,
            right_chunks[2],
            matches!(app.focus_block, FocusBlock::Table),
        )?,
        Tab::Structure => app.structure_table.draw(
            f,
            right_chunks[2],
            matches!(app.focus_block, FocusBlock::Table),
        )?,
    }
    if let Some(err) = app.error.clone() {
        draw_error_popup(f, err)?;
    }
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

pub fn common_nav(key: Key) -> Option<MoveSelection> {
    if key == Key::Char('j') {
        Some(MoveSelection::Down)
    } else if key == Key::Char('k') {
        Some(MoveSelection::Up)
    } else if key == Key::PageUp {
        Some(MoveSelection::PageUp)
    } else if key == Key::PageDown {
        Some(MoveSelection::PageDown)
    } else if key == Key::Char('l') {
        Some(MoveSelection::Right)
    } else if key == Key::Char('h') {
        Some(MoveSelection::Left)
    } else {
        None
    }
}
