use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style},
    widgets::*,
    text::{Span, Line},
    Frame,
};

use eyre::{Result, eyre};

use super::actions::Actions;
use crate::{app::{App, AppState}, utils::key_display::KeyEventWrapper};

pub fn draw<B>(frame: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let size = frame.size();

    // Body & Help
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(34)])
        .split(size);

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Length(3), Constraint::Length(3), Constraint::Min(10)])
        .split(body_chunks[0]);

    let help = draw_help(app.actions());
    frame.render_widget(help, body_chunks[1]);

    // infos
    frame.render_widget(app_infos(app.state()), content_chunks[0]);
    
    // text areas
    frame.render_widget(app.state.path_text_area.widget(), content_chunks[1]);
    frame.render_widget(app.state.filter_text_area.widget(), content_chunks[2]);

    let (content_list, content_list_state) = content(app.state_mut());
    frame.render_stateful_widget(content_list, content_chunks[3], content_list_state);
}

pub fn check_size(rect: &Rect) -> Result<()> {
    if rect.width < 52 {
        return Err(eyre!("Require width >= 52, (got {})", rect.width));
    }
    if rect.height < 12 {
        return Err(eyre!("Require height >= 28, (got {})", rect.height));
    }

    Ok(())
}

fn app_infos<'a>(state: &AppState) -> Paragraph<'a> {
    let paragraph = {
        let total_space =
            state
                .entries_size
                .iter()
                .sum::<u64>();

        let total_selected_space =
            state
                .selected_entries_idx
                .iter()
                .map(|idx| state.entries_size[*idx])
                .sum::<u64>();
        
        Paragraph::new(vec![
            Line::from(Span::raw(format!(
                "Total space: {:.2}MB",
                total_space as f32 / 1000000.0
            ))),
            Line::from(Span::raw(format!(
                "Total selected space: {:.2}MB ({:.2}%)",
                total_selected_space as f32 / 1000000.0,
                total_selected_space as f32 / total_space as f32 * 100.0
            ))),
        ])
    };

    paragraph.style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain)
            .title("Infos".to_string()),
    )
}

fn format_item(item: (&walkdir::DirEntry, u64)) -> String {
    let (entry, size) = item;
    format!(
        "path : {}, size: {:.2}MB",
        entry.path().display(),
        size as f32 / 1000000.0
    )
}
fn content<'a>(state: &mut AppState) -> (List<'a>, &mut ListState) {
    (List::new(
        state.entries
            .items()
            .iter()
            .enumerate()
            .map(|(idx, e)| (e, state.entries_size[idx]))
            .map(format_item)
            .map(ListItem::new)
            .enumerate()
            .map(|(idx, item)| {
                if state.selected_entries_idx.contains(&idx) {
                    // orange
                    item.style(Style::default().fg(Color::Rgb(255, 165, 0)))
                } else {
                    item
                }
            })
            .collect::<Vec<_>>())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title(format!("Content from path {}", state.path.canonicalize().unwrap_or("Unknown".into()).display()))
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
                Cell::from(Span::styled(KeyEventWrapper(&key).to_string(), key_style)),
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
