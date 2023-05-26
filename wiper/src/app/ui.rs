use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, List, ListItem, ListState};
use tui::Frame;

use super::{actions::Actions, state::AppState};
use crate::app::App;

pub fn draw<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let size = rect.size();
    check_size(&size).unwrap();

    // Vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(10)])
        .split(size);

    // Title
    let title = draw_title();
    rect.render_widget(title, chunks[0]);

    // Body & Help
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(32)])
        .split(chunks[1]);

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(10)])
        .split(body_chunks[0]);

    let help = draw_help(app.actions());
    rect.render_widget(help, body_chunks[1]);

    // Content
    rect.render_widget(app_infos(app.is_loading(), app.state()), content_chunks[0]);

    let (content_list, content_list_state) = content(app.state_mut());
    rect.render_stateful_widget(content_list, content_chunks[1], content_list_state);


    
}

fn draw_title<'a>() -> Paragraph<'a> {
    Paragraph::new("Wiper")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        )
}

fn check_size(rect: &Rect) -> Result<(), String> {
    if rect.width < 52 {
        return Err(format!("Require width >= 52, (got {})", rect.width));
    }
    if rect.height < 28 {
        return Err(format!("Require height >= 28, (got {})", rect.height));
    }

    Ok(())
}

fn app_infos<'a>(loading: bool, state: &AppState) -> Paragraph<'a> {
    let paragraph = if loading {
        Paragraph::new("Loading...")
    } else {

        let tick_text = format!("Tick count: {}", state.counter_tick);
    
        Paragraph::new(vec![
            Spans::from(Span::raw(tick_text)),
        ])
    };

    paragraph.style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
}

fn format_item(item: &(walkdir::DirEntry, u64)) -> String {
    let (entry, size) = item;
    format!(
        "path : {}, size: {:.2}MB",
        entry.path().canonicalize().unwrap().to_str().unwrap(),
        *size as f32 / 1000000.0
    )
}
fn content<'a>(state: &mut AppState) -> (List<'a>, &mut ListState) {
    (List::new(state.entries.items().iter().map(format_item).map(ListItem::new).enumerate().map(|(idx, item)| {
        if state.selected_entries_idx.contains(&idx) {
            // orange
            item.style(Style::default().fg(Color::Rgb(255, 165, 0)))
        } else {
            item
        }
    }).collect::<Vec<_>>())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title("Content"),
        )
        // .highlight_style(Style::default().fg(Color::LightCyan))
        .highlight_symbol(">> "),
        state.entries.state_mut())
}

fn draw_help(actions: &Actions) -> Table {
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);

    let mut rows = vec![];
    for action in actions.slice().iter() {
        let mut first = true;
        for key in action.keys() {
            let help = if first {
                first = false;
                action.to_string()
            } else {
                String::from("")
            };
            let row = Row::new(vec![
                Cell::from(Span::styled(key.to_string(), key_style)),
                Cell::from(Span::styled(help, help_style)),
            ]);
            rows.push(row);
        }
    }

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title("Help"),
        )
        .widths(&[Constraint::Length(11), Constraint::Min(20)])
        .column_spacing(1)
}
